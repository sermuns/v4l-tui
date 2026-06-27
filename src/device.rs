use std::io;

use v4l::{
    Capabilities, Control, Device as V4lDevice,
    control::{Description, Value},
};

// disambiguate between internal index or outside /dev/video{}-type index
#[derive(Clone, Copy)]
pub struct VecIndex(pub usize);
#[derive(Clone, Copy)]
pub struct DeviceIndex(pub usize);

pub struct Device {
    /// /dev/video{}..
    // index: DeviceIndex,
    pub index_as_string: String,

    capabilities: Capabilities,

    possibly_locked_descriptions: Vec<PossiblyLockedDescription>,

    #[expect(clippy::struct_field_names)]
    v4l_device: V4lDevice,
}

pub enum Modification {
    Increment,
    Decrement,
    Toggle,
}

pub struct PossiblyLockedDescription {
    pub description: Description,
    pub is_locked: bool,
}

impl Device {
    pub fn new(index: DeviceIndex, v4l_device: V4lDevice) -> Self {
        let index_as_string = index.0.to_string();

        let descriptions = v4l_device.query_controls().unwrap_or_default();

        let possibly_locked_descriptions = descriptions
            .into_iter()
            .map(|description| {
                let is_locked = v4l_device.control(&description).is_err();
                PossiblyLockedDescription {
                    description,
                    is_locked,
                }
            })
            .collect();

        Self {
            // index,
            index_as_string,
            capabilities: v4l_device.query_caps().unwrap(),
            possibly_locked_descriptions,
            v4l_device,
        }
    }

    pub fn index_str(&self) -> &str {
        &self.index_as_string
    }

    pub fn name(&self) -> &str {
        &self.capabilities.card
    }

    pub fn bus(&self) -> &str {
        &self.capabilities.bus
    }

    pub fn possibly_locked_descriptions(&self) -> &[PossiblyLockedDescription] {
        &self.possibly_locked_descriptions
    }

    pub fn control(&self, desc: &Description) -> io::Result<Control> {
        self.v4l_device.control(desc)
    }

    pub fn modify_control(
        &self,
        VecIndex(i): VecIndex,
        modification: Modification,
    ) -> color_eyre::Result<()> {
        let Some(PossiblyLockedDescription {
            description,
            is_locked,
        }) = &self.possibly_locked_descriptions.get(i)
        else {
            return Ok(());
        };

        if *is_locked {
            return Ok(());
        }

        let mut control = self.control(description)?;

        match (&mut control.value, modification) {
            (Value::Integer(value), Modification::Increment) if *value < description.maximum => {
                *value += description.step.cast_signed();
            }
            (Value::Integer(value), Modification::Decrement) if *value > description.minimum => {
                *value -= description.step.cast_signed();
            }
            (Value::Boolean(value), _) => {
                *value = !*value;
            }
            _ => return Ok(()),
        }

        self.v4l_device.set_control(control)?;

        Ok(())
    }

    pub fn num_controls(&self) -> usize {
        self.possibly_locked_descriptions.len()
    }
}
