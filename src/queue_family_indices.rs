use crate::device::Device;
use ash::vk;
use std::rc::Rc;

pub struct Queues<'a> {
    pub ash_device: Rc<Device<'a>>,
    pub graphics: vk::Queue,
    pub present: vk::Queue,
}

#[derive(Default)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}
