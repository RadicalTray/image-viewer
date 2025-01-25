use ash::prelude::*;
use ash::vk;

pub struct DescriptorPool {
    pool: vk::DescriptorPool,
}

impl DescriptorPool {
    pub unsafe fn new(
        ash_device: &ash::Device,
        create_info: &vk::DescriptorPoolCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let pool = unsafe { ash_device.create_descriptor_pool(create_info, allocator)? };
        Ok(Self { pool })
    }

    pub unsafe fn cleanup(
        self,
        ash_device: &ash::Device,
        allocator: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            ash_device.destroy_descriptor_pool(self.pool, allocator);
        }
    }

    pub fn pool(&self) -> vk::DescriptorPool {
        self.pool
    }
}
