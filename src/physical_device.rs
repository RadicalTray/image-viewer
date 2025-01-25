use std::{
    error::Error,
    ffi::{CStr, c_char},
};

use ash::{khr, prelude::*, vk};

// idk if instance is really associated with this
pub struct PhysicalDevice {
    device: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn from(physical_device: vk::PhysicalDevice) -> Self {
        let device = physical_device;
        Self { device }
    }

    pub fn device(&self) -> vk::PhysicalDevice {
        self.device
    }

    pub fn support_extensions(
        &self,
        vk_instance: &ash::Instance,
        required_extension_names: &[*const c_char],
    ) -> Result<bool, Box<dyn Error>> {
        let supported_extensions = self.query_extension_properties(vk_instance)?;
        let mut supported_extension_names = Vec::with_capacity(supported_extensions.len());
        for ext in &supported_extensions {
            supported_extension_names.push(ext.extension_name_as_c_str()?);
        }
        let mut required_extension_names: Vec<&CStr> = unsafe {
            required_extension_names
                .iter()
                .map(|x| CStr::from_ptr(x.clone()))
                .collect()
        };
        required_extension_names.retain(|x| !supported_extension_names.contains(x));

        Ok(required_extension_names.len() == 0)
    }

    pub fn query_extension_properties(
        &self,
        vk_instance: &ash::Instance,
    ) -> VkResult<Vec<vk::ExtensionProperties>> {
        unsafe { vk_instance.enumerate_device_extension_properties(self.device()) }
    }

    pub fn query_queue_family_properties(
        &self,
        vk_instance: &ash::Instance,
    ) -> Vec<vk::QueueFamilyProperties> {
        unsafe { vk_instance.get_physical_device_queue_family_properties(self.device()) }
    }

    pub fn query_support_surface(
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

    pub fn create_logical_device(
        &self,
        vk_instance: &ash::Instance,
        device_create_info: &vk::DeviceCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks<'_>>,
    ) -> VkResult<ash::Device> {
        unsafe {
            vk_instance.create_device(self.device(), device_create_info, allocation_callbacks)
        }
    }

    pub fn query_features(&self, vk_instance: &ash::Instance) -> vk::PhysicalDeviceFeatures {
        unsafe { vk_instance.get_physical_device_features(self.device()) }
    }

    pub fn query_surface_capabilities(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe { surface_instance.get_physical_device_surface_capabilities(self.device(), surface) }
    }

    pub fn query_supported_surface_formats(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe { surface_instance.get_physical_device_surface_formats(self.device(), surface) }
    }

    pub fn query_supported_present_modes(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            surface_instance.get_physical_device_surface_present_modes(self.device(), surface)
        }
    }

    pub fn query_memory_properties(
        &self,
        vk_instance: &ash::Instance,
    ) -> vk::PhysicalDeviceMemoryProperties {
        unsafe { vk_instance.get_physical_device_memory_properties(self.device()) }
    }
}
