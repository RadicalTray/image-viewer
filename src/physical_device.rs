use crate::{device::Device, instance::Instance};
use ash::{khr, prelude::*, vk};
use std::{
    error::Error,
    ffi::{CStr, c_char},
    rc::Rc,
};

pub struct PhysicalDevice {
    ash_instance: Rc<Instance>,
    device: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn new(ash_instance: Rc<Instance>, physical_device: vk::PhysicalDevice) -> Self {
        let device = physical_device;
        Self {
            ash_instance,
            device,
        }
    }

    pub fn device(&self) -> vk::PhysicalDevice {
        self.device
    }

    pub unsafe fn support_extensions(
        &self,
        required_extension_names: &[*const c_char],
    ) -> Result<bool, Box<dyn Error>> {
        let supported_extensions = unsafe { self.query_extension_properties()? };
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

    pub unsafe fn query_extension_properties(&self) -> VkResult<Vec<vk::ExtensionProperties>> {
        unsafe {
            self.ash_instance
                .instance()
                .enumerate_device_extension_properties(self.device())
        }
    }

    pub fn query_queue_family_properties(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.ash_instance
                .instance()
                .get_physical_device_queue_family_properties(self.device())
        }
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

    pub unsafe fn create_logical_device<'a>(
        &self,
        create_info: &vk::DeviceCreateInfo,
        allocator: Option<&'a vk::AllocationCallbacks<'a>>,
    ) -> VkResult<Device<'a>> {
        let device = unsafe {
            self.ash_instance
                .instance()
                .create_device(self.device(), create_info, allocator)?
        };

        Ok(Device::new(
            Rc::clone(&self.ash_instance),
            device,
            allocator,
        ))
    }

    pub unsafe fn query_features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.ash_instance
                .instance()
                .get_physical_device_features(self.device())
        }
    }

    pub unsafe fn query_surface_capabilities(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe { surface_instance.get_physical_device_surface_capabilities(self.device(), surface) }
    }

    pub unsafe fn query_supported_surface_formats(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe { surface_instance.get_physical_device_surface_formats(self.device(), surface) }
    }

    pub unsafe fn query_supported_present_modes(
        &self,
        surface_instance: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            surface_instance.get_physical_device_surface_present_modes(self.device(), surface)
        }
    }

    pub unsafe fn query_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.ash_instance
                .instance()
                .get_physical_device_memory_properties(self.device())
        }
    }
}
