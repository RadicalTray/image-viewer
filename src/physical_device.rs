use std::ffi::{CStr, c_char};

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

    pub unsafe fn support_extensions(
        &self,
        vk_instance: &ash::Instance,
        required_extension_names: &[*const c_char],
    ) -> bool {
        let supported_extensions = unsafe { self.get_extension_properties(vk_instance).unwrap() };
        let supported_extension_names: Vec<&CStr> = supported_extensions
            .iter()
            .map(|x| x.extension_name_as_c_str().unwrap())
            .collect();
        let mut required_extension_names: Vec<&CStr> = unsafe {
            required_extension_names
                .iter()
                .map(|x| CStr::from_ptr(x.clone()))
                .collect()
        };
        required_extension_names.retain(|x| !supported_extension_names.contains(x));

        required_extension_names.len() == 0
    }

    pub unsafe fn get_extension_properties(
        &self,
        vk_instance: &ash::Instance,
    ) -> VkResult<Vec<vk::ExtensionProperties>> {
        unsafe { vk_instance.enumerate_device_extension_properties(self.device()) }
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
    ) -> VkResult<bool> {
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

    pub unsafe fn get_features(&self, vk_instance: &ash::Instance) -> vk::PhysicalDeviceFeatures {
        unsafe { vk_instance.get_physical_device_features(self.device()) }
    }

    pub unsafe fn get_surface_capabilities(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe { surface_instance.get_physical_device_surface_capabilities(self.device(), surface) }
    }

    pub unsafe fn get_supported_surface_formats(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe { surface_instance.get_physical_device_surface_formats(self.device(), surface) }
    }

    pub unsafe fn get_supported_present_modes(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            surface_instance.get_physical_device_surface_present_modes(self.device(), surface)
        }
    }
}
