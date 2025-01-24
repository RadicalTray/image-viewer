use ash::vk;

pub struct Instance {
    instance: ash::Instance,
}

impl Instance {
    pub unsafe fn new(
        entry: &ash::Entry,
        create_info: &vk::InstanceCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Self {
        let instance = unsafe {
            entry
                .create_instance(create_info, allocation_callbacks)
                .expect("Failed to create vulkan instance.")
        };
        Self { instance }
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}
