use ash::prelude::*;
use ash::vk;

pub struct Semaphore {
    sem: vk::Semaphore,
}

impl Semaphore {
    pub unsafe fn new(
        ash_device: &ash::Device,
        create_info: &vk::SemaphoreCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let sem = unsafe { ash_device.create_semaphore(create_info, allocator)? };

        Ok(Self { sem })
    }

    pub unsafe fn cleanup(
        self,
        ash_device: &ash::Device,
        allocator: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            ash_device.destroy_semaphore(self.sem, allocator);
        }
    }

    pub fn sem(&self) -> vk::Semaphore {
        self.sem
    }
}
