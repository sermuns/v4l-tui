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
    widgets::Block,
};
use v4l::Device;

#[derive(Default)]
pub struct App {
    quit: bool,
    devices: Vec<Device>,
    selected_device_index: Option<usize>,
    ffplay_child: Option<Child>,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let mut s = Self {
            selected_device_index: Some(1),
            ..Default::default()
        };
        s.refresh_devices()?;
        Ok(s)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;

            self.handle_keypresses()?;
        }

        Ok(())
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
            KeyCode::Char('r') => self.refresh_devices()?,
            KeyCode::Char('p') => self.start_preview()?,
            _ => (),
        }

        Ok(())
    }

    fn start_preview(&mut self) -> io::Result<()> {
        if let Some(i) = self.selected_device_index {
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

    fn refresh_devices(&mut self) -> io::Result<()> {
        self.devices.clear();

        for i in 0..10 {
            if let Ok(device) = Device::new(i) {
                self.devices.push(device);
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(concat!(" ", env!("CARGO_PKG_NAME"), " "))
            .title_alignment(HorizontalAlignment::Center);
        (&block).render(area, buf);

        let inner_area = block.inner(area);
    }
}
