use std::time::{Duration, Instant};

use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

pub struct Notification {
    message: String,
    dead_by: Instant,
}

const DURATION: Duration = Duration::from_secs(3);

impl Notification {
    pub fn new(message: String) -> Self {
        Self {
            message,
            dead_by: Instant::now() + DURATION,
        }
    }
    pub fn is_dead(&self) -> bool {
        Instant::now() >= self.dead_by
    }
    pub fn len(&self) -> usize {
        self.message.len()
    }
}

impl Widget for &Notification {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear.render(area, buf);
        Paragraph::new(self.message.as_str())
            .block(Block::bordered().border_style(Style::new().green()))
            .render(area, buf);
    }
}
