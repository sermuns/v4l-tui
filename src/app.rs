use std::{
    io,
    process::{Child, Command, Stdio},
    time::Duration,
};

use ratatui::{
    DefaultTerminal,
    crossterm::{
        self,
        event::{KeyCode, KeyModifiers},
    },
    prelude::*,
    widgets::{Block, Row, Table, TableState},
};
use v4l::Device;

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
    DeviceConfig(Device),
}

enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let mut app = Self::default();
        app.refresh_devices();
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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::bordered()
            .title(concat!(" ", env!("CARGO_PKG_NAME"), " ").bold())
            .title_alignment(HorizontalAlignment::Center);
        frame.render_widget(&block, area);

        let inner_area = block.inner(area);

        const HEADER: [&str; 3] = ["Index", "Card", "Bus"];
        const WIDTHS: [Constraint; HEADER.len()] = [
            Constraint::Length(HEADER[0].len() as u16),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];

        let rows = self.devices.iter().enumerate().filter_map(|(i, device)| {
            let device = device.as_ref()?;

            let row = if let Ok(caps) = device.query_caps() {
                Row::new([i.to_string(), caps.card, caps.bus])
            } else {
                Row::new([i.to_string(), "N/A".to_owned(), "N/A".to_owned()])
            }
            .style(
                if self
                    .devices_table_state
                    .selected()
                    .is_some_and(|selected_index| selected_index == i)
                {
                    Style::default().reversed()
                } else {
                    Style::default()
                },
            );
            Some(row)
        });

        frame.render_stateful_widget(
            Table::new(rows, WIDTHS).header(Row::new(HEADER)),
            inner_area,
            &mut self.devices_table_state,
        );

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
            _ => (),
        }

        Ok(())
    }

    fn perform_action(&mut self, action: Action) {
        if self.devices_table_state.selected().is_none() {
            self.devices_table_state.select_first();
            return;
        }
        match self.focused_block {
            FocusedBlock::DevicesTable => match action {
                Action::MoveUp => {
                    self.devices_table_state.select_previous();
                }
                Action::MoveDown => {
                    self.devices_table_state.select_next();
                }
                _ => (),
            },
            FocusedBlock::DeviceConfig(..) => todo!(),
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
            self.devices[i] = Device::new(i).ok();
        }
    }
}
