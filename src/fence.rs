use ash::prelude::*;
use ash::vk;

pub struct Fence {
    fence: vk::Fence,
}

impl Fence {
    pub unsafe fn new(
        ash_device: &ash::Device,
        create_info: &vk::FenceCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let fence = unsafe { ash_device.create_fence(create_info, allocator)? };
        Ok(Self { fence })
    }

    pub unsafe fn cleanup(
        self,
        ash_device: &ash::Device,
        allocator: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            ash_device.destroy_fence(self.fence, allocator);
        }
    }

    pub fn fence(&self) -> vk::Fence {
        self.fence
    }
}
