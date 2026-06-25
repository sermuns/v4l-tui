use std::io;

use ratatui::{
    DefaultTerminal,
    crossterm::{self, event::KeyCode},
    prelude::*,
    text::ToLine,
    widgets::Block,
};
use v4l::Device;

pub struct App {
    running: bool,
    devices: Vec<Device>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            devices: Vec::new(),
        }
    }
}

impl App {
    pub fn new() -> io::Result<Self> {
        let mut s = Self::default();
        s.refresh_devices()?;
        Ok(s)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
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
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('r') => self.refresh_devices()?,
            _ => (),
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

        // for (i, device) in self.devices.iter().enumerate() {
        // }
        self.devices.len().to_line().render(inner_area, buf);
    }
}
