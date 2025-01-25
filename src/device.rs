pub struct Device {
    device: ash::Device,
}

impl Device {
    pub fn from(device: ash::Device) -> Self {
        Self { device }
    }

    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    pub unsafe fn cleanup(self, allocator: Option<&ash::vk::AllocationCallbacks>) {
        unsafe {
            self.device.destroy_device(allocator);
        }
    }
}
