use ash::prelude::*;
use ash::vk;

pub struct Instance {
    instance: ash::Instance,
}

impl Instance {
    pub unsafe fn new(
        ash_entry: &ash::Entry,
        create_info: &vk::InstanceCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let instance = unsafe { ash_entry.create_instance(create_info, allocator)? };
        Ok(Self { instance })
    }

    pub unsafe fn cleanup(self, allocator: Option<&vk::AllocationCallbacks>) {
        unsafe {
            self.instance.destroy_instance(allocator);
        }
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }
}
