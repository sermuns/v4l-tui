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

impl FocusedBlock {
    // TODO:
    pub fn help_text(&self) -> &str {
        let directional = " hjkl / arrow keys | ENTER / spacebar";
        // match self {
        //     Self::DevicesTable => {
        //     }
        // }
        directional
    }
}
