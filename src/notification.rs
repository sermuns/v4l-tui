use std::time::{Duration, Instant};

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph, Wrap},
};

pub enum Severity {
    Info,
    Error,
}

pub struct Notification {
    message: String,
    severity: Severity,
    dead_by: Instant,
}

const DURATION: Duration = Duration::from_secs(3);

impl Notification {
    pub fn new(message: String, severity: Severity) -> Self {
        Self {
            message,
            severity,
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
        let border_style = match self.severity {
            Severity::Info => Style::new().green(),
            Severity::Error => Style::new().red(),
        };
        Paragraph::new(self.message.as_str())
            .wrap(Wrap { trim: true })
            .block(
                Block::bordered()
                    .border_style(border_style)
                    .border_type(BorderType::Thick),
            )
            .render(area, buf);
    }
}
