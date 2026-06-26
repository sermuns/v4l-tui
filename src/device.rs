use std::io;

use v4l::{Capabilities, Control, Device as V4lDevice, control::Description};

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

    control_descriptions: Vec<Description>,

    v4l_device: V4lDevice,
}

impl Device {
    pub fn new(index: DeviceIndex, v4l_device: V4lDevice) -> Self {
        let index_as_string = index.0.to_string();

        Self {
            // index,
            index_as_string,
            capabilities: v4l_device.query_caps().unwrap(),
            control_descriptions: v4l_device.query_controls().unwrap(),
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

    pub fn descriptions(&self) -> &[Description] {
        &self.control_descriptions
    }

    pub fn control(&self, id: u32) -> io::Result<Control> {
        self.v4l_device.control(id)
    }

    pub fn num_controls(&self) -> usize {
        self.control_descriptions.len()
    }
}
