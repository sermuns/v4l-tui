use std::{
    io,
    process::{Child, Command, Stdio},
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

#[derive(Default)]
pub struct App {
    quit: bool,
    devices: [Option<Device>; 10],
    devices_table_state: TableState,
    ffplay_child: Option<Child>,
    focused_block: FocusedBlock,
}

#[derive(Default)]
enum FocusedBlock {
    #[default]
    DevicesTable,
}

enum MoveDir {
    Up,
    Down,
    Left,
    Right,
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
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let block = Block::bordered()
            .title(concat!(" ", env!("CARGO_PKG_NAME"), " "))
            .title_alignment(HorizontalAlignment::Center);
        frame.render_widget(&block, frame.area());

        let inner_area = block.inner(frame.area());

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
                    Style::default().add_modifier(Modifier::REVERSED)
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
    }

    fn handle_keypresses(&mut self) -> io::Result<()> {
        use crossterm::event::Event;
        let Event::Key(key_event) = crossterm::event::read()? else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.quit = true
            }
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('r') => self.refresh_devices(),
            KeyCode::Char('p') => self.start_preview()?,
            KeyCode::Char('k') | KeyCode::Up => self.move_focus(MoveDir::Up),
            KeyCode::Char('j') | KeyCode::Down => self.move_focus(MoveDir::Down),
            KeyCode::Char('h') | KeyCode::Left => self.move_focus(MoveDir::Left),
            KeyCode::Char('l') | KeyCode::Right => self.move_focus(MoveDir::Right),
            _ => (),
        }

        Ok(())
    }

    fn move_focus(&mut self, dir: MoveDir) {
        if self.devices_table_state.selected().is_none() {
            self.devices_table_state.select_first();
            return;
        }
        match self.focused_block {
            FocusedBlock::DevicesTable => match dir {
                MoveDir::Up => {
                    self.devices_table_state.select_previous();
                }
                MoveDir::Down => {
                    self.devices_table_state.select_next();
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
            self.devices[i] = Device::new(i).ok();
        }
    }
}
