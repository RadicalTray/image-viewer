use ash::prelude::*;
use ash::vk;

pub struct CommandPool {
    pool: vk::CommandPool,
}

impl CommandPool {
    pub unsafe fn new(
        ash_device: &ash::Device,
        create_info: &vk::CommandPoolCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let pool = unsafe { ash_device.create_command_pool(create_info, allocator)? };
        Ok(Self { pool })
    }

    pub unsafe fn cleanup(
        self,
        ash_device: &ash::Device,
        allocator: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            ash_device.destroy_command_pool(self.pool, allocator);
        }
    }

    pub fn pool(&self) -> vk::CommandPool {
        self.pool
    }
}
