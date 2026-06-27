use crate::{
    app::App,
    device::{Modification, VecIndex},
    focused_block::FocusedBlock,
};

pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Confirm,
    Cancel,
}

impl App {
    pub fn perform_action(&mut self, action: Action) -> color_eyre::Result<()> {
        if !matches!(action, Action::Cancel) && self.devices_table_state.selected().is_none() {
            self.devices_table_state.select_first();
            return Ok(());
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
                    self.focused_block = FocusedBlock::DeviceConfig {
                        device_index: VecIndex(i),
                        selected_control_row: 0,
                    };
                }
                _ => (),
            },
            FocusedBlock::DeviceConfig {
                device_index,
                ref mut selected_control_row,
            } => {
                let device = &self.devices[device_index.0];
                match action {
                    Action::Cancel => self.focused_block = FocusedBlock::DevicesTable,
                    Action::MoveDown => {
                        if *selected_control_row < device.num_controls().saturating_sub(1) {
                            *selected_control_row += 1;
                        } else {
                            *selected_control_row = 0;
                        }
                    }
                    Action::MoveUp => {
                        if *selected_control_row > 0 {
                            *selected_control_row -= 1;
                        } else {
                            *selected_control_row = device.num_controls().saturating_sub(1);
                        }
                    }
                    Action::MoveRight => device
                        .modify_control(VecIndex(*selected_control_row), Modification::Increment)?,

                    Action::MoveLeft => device
                        .modify_control(VecIndex(*selected_control_row), Modification::Decrement)?,
                    Action::Confirm => device
                        .modify_control(VecIndex(*selected_control_row), Modification::Toggle)?,
                }
            }
        }

        Ok(())
    }
}
