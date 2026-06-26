use ratatui::prelude::*;

use crate::device::VecIndex;

#[derive(Default)]
pub enum FocusedBlock {
    #[default]
    DevicesTable,
    DeviceConfig {
        device_index: VecIndex,
        selected_control_row: usize,
    },
}

macro_rules! more_help_str {
    ($x:literal) => {
        concat!(" up: k/\u{2191}, down: j/\u{2193}, ", $x, " ")
    };
}

impl FocusedBlock {
    // TODO:
    pub fn help_text(&self) -> &'static str {
        match self {
            Self::DevicesTable => more_help_str!("confirm: enter/space, quit: q"),
            Self::DeviceConfig { .. } => {
                more_help_str!("dec: h/\u{2190}, inc: l/\u{2192}, toggle: enter/space, back: esc")
            }
        }
    }
}
