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
    widgets::{Block, Gauge, Padding, Row, Table, TableState},
};
use v4l::{
    Device,
    control::{Description, Value},
};

use crate::notification::{Notification, Severity};

#[derive(Default)]
pub struct App {
    quit: bool,
    devices: [Option<Device>; 10],
    devices_table_state: TableState,
    ffplay_child: Option<Child>,
    focused_block: FocusedBlock,
    notification: Option<Notification>,
}

#[derive(Default)]
enum FocusedBlock {
    #[default]
    DevicesTable,
    DeviceConfig {
        device_index: usize,
        controls: Vec<Description>,
        selected_row: usize,
    },
}

enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Confirm,
    Cancel,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let mut app = Self {
            devices_table_state: TableState::new().with_selected(1),
            ..Default::default()
        };

        app.refresh_devices();

        app.focused_block = FocusedBlock::DeviceConfig {
            device_index: 1,
            controls: app.devices[1].as_ref().unwrap().query_controls().unwrap(),
            selected_row: 0,
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

        let rows = self.devices.iter().enumerate().map(|(i, device)| {
            if let Some(device) = &device
                && let Ok(caps) = device.query_caps()
            {
                Row::new([i.to_string(), caps.card, caps.bus])
            } else {
                Row::new([i.to_string(), "N/A".to_owned(), "N/A".to_owned()].map(|s| s.dim()))
            }
        });

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
        device_index: usize,
        controls: &[Description],
        selected_row: usize,
    ) {
        let device = self.devices[device_index].as_ref().unwrap();

        let horizontal = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Length(30),
            Constraint::Fill(1),
        ])
        .spacing(1);

        let row_constraints = (0..controls.len()).map(|_| Constraint::Length(1));
        let vertical = Layout::vertical(row_constraints).spacing(1);

        let cells = area
            .layout_vec(&vertical)
            .into_iter()
            .flat_map(|row| row.layout_vec(&horizontal));

        for ((i, control), mut cells_in_row) in controls.iter().enumerate().zip(&cells.chunks(3)) {
            let selector_area = cells_in_row.next().unwrap();
            if selected_row == i {
                frame.render_widget("->".bold().yellow(), selector_area);
            }

            let name_area = cells_in_row.next().unwrap();
            let gauge_area = cells_in_row.next().unwrap();

            let Ok(current_control) = device.control(control.id) else {
                continue;
            };
            let Value::Integer(value) = current_control.value else {
                continue;
            };
            let ratio =
                (value - control.minimum) as f64 / (control.maximum - control.minimum) as f64;

            frame.render_widget(control.name.as_str(), name_area);
            frame.render_widget(Gauge::default().ratio(ratio), gauge_area);
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::bordered()
            .title(concat!(" ", env!("CARGO_PKG_NAME"), " ").bold().dim())
            .padding(Padding::proportional(1))
            .title_alignment(HorizontalAlignment::Center);
        frame.render_widget(&block, area);

        let inner_area = block.inner(area);

        match self.focused_block {
            FocusedBlock::DevicesTable => self.draw_devices_table(frame, inner_area),
            FocusedBlock::DeviceConfig {
                device_index,
                ref controls,
                selected_row,
            } => self.draw_device_config(frame, inner_area, device_index, controls, selected_row),
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
            KeyCode::Char('p') => self.start_preview()?,
            KeyCode::Char('k') | KeyCode::Up => self.perform_action(Action::MoveUp),
            KeyCode::Char('j') | KeyCode::Down => self.perform_action(Action::MoveDown),
            KeyCode::Char('h') | KeyCode::Left => self.perform_action(Action::MoveLeft),
            KeyCode::Char('l') | KeyCode::Right => self.perform_action(Action::MoveRight),
            KeyCode::Char(' ') | KeyCode::Enter => self.perform_action(Action::Confirm),
            KeyCode::Esc | KeyCode::Backspace => self.perform_action(Action::Cancel),
            _ => (),
        }

        Ok(())
    }

    fn perform_action(&mut self, action: Action) {
        if !matches!(action, Action::Cancel) && self.devices_table_state.selected().is_none() {
            self.devices_table_state.select_first();
            return;
        }
        match self.focused_block {
            FocusedBlock::DevicesTable => match action {
                Action::MoveUp
                    if let Some(selected_index) = self.devices_table_state.selected() =>
                {
                    if selected_index > 0 {
                        self.devices_table_state.select_previous();
                    } else {
                        self.devices_table_state.select_last();
                    }
                }
                Action::MoveDown
                    if let Some(selected_index) = self.devices_table_state.selected() =>
                {
                    if selected_index < self.devices.len() - 1 {
                        self.devices_table_state.select_next();
                    } else {
                        self.devices_table_state.select_first();
                    }
                }
                Action::Confirm if let Some(i) = self.devices_table_state.selected() => {
                    if let Some(device) = &self.devices[i] {
                        self.focused_block = FocusedBlock::DeviceConfig {
                            device_index: i,
                            controls: device.query_controls().unwrap(),
                            selected_row: 0,
                        };
                    } else {
                        self.notification = Some(Notification::new(
                            format!("Device /dev/video{} not found", i),
                            Severity::Error,
                        ));
                    }
                }
                Action::Cancel => self.devices_table_state.select(None),
                _ => (),
            },
            FocusedBlock::DeviceConfig {
                ref mut selected_row,
                ref controls,
                ..
            } => match action {
                Action::Cancel => self.focused_block = FocusedBlock::DevicesTable,
                Action::MoveDown => {
                    if *selected_row < controls.len() - 1 {
                        *selected_row += 1;
                    } else {
                        *selected_row = 0;
                    }
                }
                Action::MoveUp => {
                    if *selected_row > 0 {
                        *selected_row -= 1;
                    } else {
                        *selected_row = controls.len() - 1;
                    }
                }
                _ => (),
            },
        }
    }

    fn start_preview(&mut self) -> io::Result<()> {
        if let Some(i) = self.devices_table_state.selected() {
            self.ffplay_child = Some(
                Command::new("ffplay")
                    .arg(format!("/dev/video{}", i))
                    .stderr(Stdio::null())
                    .stdout(Stdio::null())
                    .spawn()?,
            );
        }

        Ok(())
    }

    fn refresh_devices(&mut self) {
        for i in 0..self.devices.len() {
            let Ok(device) = Device::new(i) else {
                self.devices[i] = None;
                continue;
            };

            // remove "Metadata Capture" devices. we only want "Video Capture" devices
            // https://askubuntu.com/a/1229301
            if device
                .query_caps()
                .unwrap()
                .capabilities
                .contains(v4l::capability::Flags::META_CAPTURE)
            {
                self.devices[i] = None;
                continue;
            }

            self.devices[i] = Some(device);
        }
    }
}
