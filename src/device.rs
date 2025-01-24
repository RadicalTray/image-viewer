use std::rc::Rc;

use crate::instance::Instance;
use ash::vk;

pub struct Device<'a> {
    ash_instance: Rc<Instance>,
    device: ash::Device,
    allocator: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> Device<'a> {
    pub fn new(
        ash_instance: Rc<Instance>,
        device: ash::Device,
        allocator: Option<&'a vk::AllocationCallbacks<'a>>,
    ) -> Self {
        Self {
            ash_instance,
            device,
            allocator,
        }
    }

    pub fn device(&self) -> &ash::Device {
        &self.device
    }
}

impl<'a> Drop for Device<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(self.allocator);
        }
    }
}
