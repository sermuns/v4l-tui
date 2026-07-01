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
    style::Styled,
    widgets::{Block, Borders, Clear, Gauge, Padding, Paragraph, Row, Table, TableState},
};
use v4l::{Device as V4lDevice, control::Value};

use crate::{
    action::Action,
    device::{Device, DeviceIndex, PossiblyLockedDescription, VecIndex},
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
    pub fn new() -> Self {
        let mut app = Self::default();
        app.refresh_devices();
        app.devices_table_state.select_first();

        // app.focused_block = FocusedBlock::DeviceConfig {
        //     device_index: VecIndex(0),
        //     selected_control_row: 0,
        // };

        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.quit {
            terminal.draw(|frame| self.draw(frame))?;

            // self.handle_keypresses()?;
            if let Err(e) = self.handle_keypresses() {
                self.notification = Some(Notification::new(format!("{:?}", e), Severity::Error));
            }

            self.handle_ffplay_child()?;
            if self
                .notification
                .as_ref()
                .is_some_and(Notification::is_dead)
            {
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

        let [device_name_area, controls_area] =
            area.layout(&Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1));
        frame.render_widget(device.name().dim(), device_name_area);

        let controls =
            device
                .possibly_locked_descriptions()
                .iter()
                .map(|possibly_locked_description| {
                    let control = device
                        .control(&possibly_locked_description.description)
                        .ok();
                    (possibly_locked_description, control.map(|c| c.value))
                });

        let row_constraints = (0..device.num_controls()).map(|_| Constraint::Length(1));
        let vertical = Layout::vertical(row_constraints).spacing(1);

        #[expect(clippy::items_after_statements)]
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

        for ((i, (possibly_locked_description, maybe_value)), mut cells_in_row) in controls
            .enumerate()
            .zip(&cells.chunks(HORIZONTAL_CONSTRAINTS.len()))
        {
            let selector_area = cells_in_row.next().unwrap();
            if selected_row == i {
                frame.render_widget("->".bold().yellow(), selector_area);
            }

            let PossiblyLockedDescription {
                is_locked,
                description,
            } = possibly_locked_description;

            let style = if *is_locked {
                Style::new().dim().crossed_out()
            } else {
                Style::new()
            };

            let control_name_area = cells_in_row.next().unwrap();
            frame.render_widget(
                description.name.as_str().set_style(style),
                control_name_area,
            );

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
                    frame.render_widget(value_string.set_style(style), value_area);

                    let ratio = (value - description.minimum) as f64
                        / (description.maximum - description.minimum) as f64;
                    frame.render_widget(
                        Gauge::default().ratio(ratio).set_style(style),
                        visualisation_area,
                    );
                }
                Value::Boolean(value) => {
                    let value_string = value.to_string();
                    frame.render_widget(value_string.as_str().set_style(style), value_area);

                    let ratio = if value { 1.0 } else { 0.0 };
                    frame.render_widget(
                        Gauge::default()
                            .ratio(ratio)
                            .label(value_string)
                            .set_style(style),
                        visualisation_area,
                    );
                }
                _ => (),
            }
        }
    }

    fn draw_preview_status(&self, frame: &mut Frame, area: Rect) {
        let (line, foreground_color) = if self.ffplay_child.is_some() {
            (
                Line::from_iter(["Currently previewing! Press 'p' again to stop."]),
                Color::Red,
            )
        } else {
            (
                Line::from_iter(["Press 'p' to start preview."]),
                Color::Green,
            )
        };

        let paragraph = Paragraph::new(line)
            .centered()
            .block(Block::new().borders(Borders::TOP))
            .fg(foreground_color);

        frame.render_widget(paragraph, area);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::bordered()
            .title(concat!(" ", env!("CARGO_PKG_NAME"), " ").bold().dim())
            .title_bottom(self.focused_block.help_text())
            .border_style(Style::new().yellow())
            .padding(Padding::proportional(1))
            .title_alignment(HorizontalAlignment::Center);
        frame.render_widget(&block, area);

        let inner_area = block.inner(area);

        let [body_area, preview_status_area] = inner_area.layout(&Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(2),
        ]));

        match self.focused_block {
            FocusedBlock::DevicesTable => {
                if self.devices.is_empty() {
                    frame.render_widget("No camera devices detected.", body_area);
                } else {
                    self.draw_devices_table(frame, body_area);
                }
            }
            FocusedBlock::DeviceConfig {
                device_index,
                selected_control_row,
            } => self.draw_device_config(frame, body_area, device_index, selected_control_row),
        }

        self.draw_preview_status(frame, preview_status_area);

        if let Some(notification) = &self.notification {
            let notification_area = area.centered(Constraint::Fill(1), Constraint::Max(20));
            frame.render_widget(Clear, notification_area);
            frame.render_widget(notification, notification_area);
        }
    }

    fn handle_keypresses(&mut self) -> anyhow::Result<()> {
        if !crossterm::event::poll(Duration::from_millis(500))? {
            return Ok(());
        }

        let crossterm::event::Event::Key(key_event) = crossterm::event::read()? else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true;
            }
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('r') => self.refresh_devices(),
            KeyCode::Char('p') if let Some(mut child) = self.ffplay_child.take() => {
                child.kill()?;
            }
            KeyCode::Char('p') if let Some(i) = self.devices_table_state.selected() => {
                self.start_preview(VecIndex(i))?;
            }
            KeyCode::Char('k') | KeyCode::Up => self.perform_action(Action::MoveUp)?,
            KeyCode::Char('j') | KeyCode::Down => self.perform_action(Action::MoveDown)?,
            KeyCode::Char('-' | 'h') | KeyCode::Left => {
                self.perform_action(Action::MoveLeft)?;
            }
            KeyCode::Char('+' | 'l') | KeyCode::Right => {
                self.perform_action(Action::MoveRight)?;
            }
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
            if let Ok(caps) = v4l_device.query_caps()
                && caps
                    .capabilities
                    .contains(v4l::capability::Flags::META_CAPTURE)
            {
                continue;
            }

            self.devices.push(Device::new(DeviceIndex(i), v4l_device));
        }
    }
}
