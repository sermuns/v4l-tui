use crate::device::VecIndex;

use const_format::formatcp;
use unicode_consts::arrows::{DOWNWARDS_ARROW, LEFTWARDS_ARROW, RIGHTWARDS_ARROW, UPWARDS_ARROW};

#[derive(Default)]
pub enum FocusedBlock {
    #[default]
    DevicesTable,
    DeviceConfig {
        device_index: VecIndex,
        selected_control_row: usize,
    },
}

const UP_DOWN_HELP: &str = formatcp!("up: k/{UPWARDS_ARROW}, down: j/{DOWNWARDS_ARROW}");

impl FocusedBlock {
    // TODO:
    pub fn help_text(&self) -> &'static str {
        match self {
            Self::DevicesTable => formatcp!(" {UP_DOWN_HELP}, confirm: enter/space, quit: q"),
            Self::DeviceConfig { .. } => {
                formatcp!(
                    " {UP_DOWN_HELP}, dec: -/h/{LEFTWARDS_ARROW}, inc: +/l/{RIGHTWARDS_ARROW}, toggle: enter/space, back: esc "
                )
            }
        }
    }
}
