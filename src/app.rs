use std::{
    io,
    process::{Child, Command, Stdio},
    time::Duration,
};

use itertools::Itertools;
use ratatui::{
    DefaultTerminal,
    crossterm::{
        self,
        event::{KeyCode, KeyModifiers},
    },
    prelude::*,
    text::ToLine,
    widgets::{Block, Gauge, Padding, Row, Table, TableState},
};
use v4l::{Device as V4lDevice, control::Value};

use crate::{
    action::Action,
    device::{Device, DeviceIndex, VecIndex},
    focused_block::FocusedBlock,
    notification::{Notification, Severity},
};

const MAX_DEVICE_INDEX: usize = 10;

#[derive(Default)]
pub struct App {
    pub quit: bool,
    pub devices: Vec<Device>,
    pub devices_table_state: TableState,
    pub ffplay_child: Option<Child>,
    pub focused_block: FocusedBlock,
    pub notification: Option<Notification>,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let mut app = Self::default();

        app.refresh_devices();

        app.devices_table_state.select_first();

        app.focused_block = FocusedBlock::DeviceConfig {
            device_index: VecIndex(0),
            selected_control_row: 0,
        };

        Ok(app)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.draw(frame))?;

            self.handle_keypresses()?;
            self.handle_ffplay_child()?;
            if self.notification.as_ref().is_some_and(|n| n.is_dead()) {
                self.notification = None;
            }
        }

        if let Some(child) = &mut self.ffplay_child {
            let _ = child.kill();
        }

        Ok(())
    }

    fn handle_ffplay_child(&mut self) -> io::Result<()> {
        if let Some(child) = &mut self.ffplay_child
            && child.try_wait()?.is_some()
        {
            self.ffplay_child = None;
            self.notification = Some(Notification::new(
                String::from("Preview closed"),
                Severity::Info,
            ));
        }

        Ok(())
    }

    fn draw_devices_table(&mut self, frame: &mut Frame, area: Rect) {
        const HEADER: [&str; 3] = ["Index", "Card", "Bus"];
        const WIDTHS: [Constraint; HEADER.len()] = [
            Constraint::Length(HEADER[0].len() as u16 + 1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];

        let rows = self
            .devices
            .iter()
            .map(|device| Row::new([device.index_str(), device.name(), device.bus()]));

        frame.render_stateful_widget(
            Table::new(rows, WIDTHS)
                .header(Row::new(HEADER).bold().yellow())
                .row_highlight_style(Style::default().on_dark_gray())
                .highlight_symbol("-> "),
            area,
            &mut self.devices_table_state,
        );
    }

    fn draw_device_config(
        &self,
        frame: &mut Frame,
        area: Rect,
        device_index: VecIndex,
        selected_row: usize,
    ) {
        let device = &self.devices[device_index.0];

        let [device_name_area, controls_area] = area
            .layout(&Layout::horizontal([Constraint::Length(1), Constraint::Fill(1)]).spacing(1));
        frame.render_widget(device.name().to_line().centered(), device_name_area);

        let integer_controls = device.descriptions().iter().map(|description| {
            let control = device.control(description.id).ok();
            (description, control.map(|c| c.value))
        });

        // FIXME: this is a mess, and stupid to clone
        let row_constraints = (0..integer_controls.clone().count()).map(|_| Constraint::Length(1));
        let vertical = Layout::vertical(row_constraints).spacing(1);

        const HORIZONTAL_CONSTRAINTS: [Constraint; 4] = [
            Constraint::Length(2),
            Constraint::Length(26),
            Constraint::Length(20),
            Constraint::Fill(1),
        ];
        let horizontal = Layout::horizontal(HORIZONTAL_CONSTRAINTS).spacing(1);

        let cells = controls_area
            .layout_vec(&vertical)
            .into_iter()
            .flat_map(|row| row.layout_vec(&horizontal));

        for ((i, (description, maybe_value)), mut cells_in_row) in integer_controls
            .enumerate()
            .zip(&cells.chunks(HORIZONTAL_CONSTRAINTS.len()))
        {
            let selector_area = cells_in_row.next().unwrap();
            if selected_row == i {
                frame.render_widget("->".bold().yellow(), selector_area);
            }

            let control_name_area = cells_in_row.next().unwrap();
            frame.render_widget(description.name.as_str(), control_name_area);

            let Some(value) = maybe_value else {
                continue;
            };

            let value_area = cells_in_row.next().unwrap();
            let visualisation_area = cells_in_row.next().unwrap();

            match value {
                Value::Integer(value) => {
                    // FIXME: alloc...
                    let value_string = format!(
                        "{} ({}..{})",
                        value, description.minimum, description.maximum
                    );
                    frame.render_widget(value_string, value_area);

                    let ratio = (value - description.minimum) as f64
                        / (description.maximum - description.minimum) as f64;
                    frame.render_widget(Gauge::default().ratio(ratio), visualisation_area);
                }
                Value::Boolean(value) => {
                    let value_string = value.to_string();
                    frame.render_widget(value_string.as_str(), value_area);

                    let ratio = if value { 1.0 } else { 0.0 };
                    frame.render_widget(
                        Gauge::default().ratio(ratio).label(value_string),
                        visualisation_area,
                    );
                }
                _ => (),
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::bordered()
            .title(concat!(" ", env!("CARGO_PKG_NAME"), " ").bold().dim())
            .title_bottom(self.focused_block.help_text())
            .padding(Padding::proportional(1))
            .title_alignment(HorizontalAlignment::Center);
        frame.render_widget(&block, area);

        let inner_area = block.inner(area);

        match self.focused_block {
            FocusedBlock::DevicesTable => self.draw_devices_table(frame, inner_area),
            FocusedBlock::DeviceConfig {
                device_index,
                selected_control_row,
            } => self.draw_device_config(frame, inner_area, device_index, selected_control_row),
        }

        if let Some(notification) = &self.notification {
            let notification_area = Rect {
                x: area.x,
                y: area.bottom() - 3,
                width: notification.len() as u16 + 2,
                height: 3,
            };
            frame.render_widget(notification, notification_area);
        }
    }

    fn handle_keypresses(&mut self) -> io::Result<()> {
        if !crossterm::event::poll(Duration::from_millis(100))? {
            return Ok(());
        }

        let crossterm::event::Event::Key(key_event) = crossterm::event::read()? else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true
            }
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('r') => self.refresh_devices(),
            KeyCode::Char('p') if let Some(i) = self.devices_table_state.selected() => {
                self.start_preview(VecIndex(i))?
            }
            KeyCode::Char('k') | KeyCode::Up => self.perform_action(Action::MoveUp)?,
            KeyCode::Char('j') | KeyCode::Down => self.perform_action(Action::MoveDown)?,
            KeyCode::Char('h') | KeyCode::Left => self.perform_action(Action::MoveLeft)?,
            KeyCode::Char('l') | KeyCode::Right => self.perform_action(Action::MoveRight)?,
            KeyCode::Char(' ') | KeyCode::Enter => self.perform_action(Action::Confirm)?,
            KeyCode::Esc | KeyCode::Backspace => self.perform_action(Action::Cancel)?,
            _ => (),
        }

        Ok(())
    }

    fn start_preview(&mut self, VecIndex(i): VecIndex) -> io::Result<()> {
        let index_str = self.devices[i].index_str();
        self.ffplay_child = Some(
            Command::new("ffplay")
                .arg(format!("/dev/video{}", index_str))
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .spawn()?,
        );

        Ok(())
    }

    fn refresh_devices(&mut self) {
        self.devices.clear();

        for i in 0..MAX_DEVICE_INDEX {
            let Ok(v4l_device) = V4lDevice::new(i) else {
                continue;
            };

            // remove "Metadata Capture" devices. we only want "Video Capture" devices
            // https://askubuntu.com/a/1229301
            if v4l_device
                .query_caps()
                .unwrap()
                .capabilities
                .contains(v4l::capability::Flags::META_CAPTURE)
            {
                continue;
            }

            self.devices.push(Device::new(DeviceIndex(i), v4l_device));
        }
    }
}
