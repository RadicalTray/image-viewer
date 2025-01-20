use ash::{khr, prelude::*, vk};

// idk if instance is really associated with this
pub struct PhysicalDevice {
    device: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn new(physical_device: vk::PhysicalDevice) -> Self {
        let device = physical_device;
        Self { device }
    }

    pub fn device(&self) -> vk::PhysicalDevice {
        self.device
    }

    pub unsafe fn get_queue_family_properties(
        &self,
        vk_instance: &ash::Instance,
    ) -> Vec<vk::QueueFamilyProperties> {
        unsafe { vk_instance.get_physical_device_queue_family_properties(self.device()) }
    }

    pub unsafe fn support_surface(
        &self,
        surface_instance: &khr::surface::Instance,
        queue_family_index: u32,
        surface: vk::SurfaceKHR,
    ) -> std::result::Result<bool, ash::vk::Result> {
        unsafe {
            surface_instance.get_physical_device_surface_support(
                self.device,
                queue_family_index,
                surface,
            )
        }
    }

    pub unsafe fn create_logical_device(
        &self,
        vk_instance: &ash::Instance,
        device_create_info: &vk::DeviceCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<ash::Device> {
        unsafe {
            vk_instance.create_device(self.device(), device_create_info, allocation_callbacks)
        }
    }
}
