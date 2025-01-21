use ash::{khr, prelude::*, vk};

// lifetime is 100% not working, but everything is working because we're using no allocation_callbacks
pub struct Swapchain<'a> {
    device: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    allocation_callbacks: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> Swapchain<'a> {
    pub fn new(
        vk_instance: &ash::Instance,
        vk_device: &ash::Device,
        swapchain_info: &vk::SwapchainCreateInfoKHR,
        allocation_callbacks: Option<&'a vk::AllocationCallbacks<'_>>,
    ) -> VkResult<Self> {
        let format = swapchain_info.image_format;
        let extent = swapchain_info.image_extent;
        let device = khr::swapchain::Device::new(vk_instance, vk_device);
        let swapchain = unsafe { device.create_swapchain(swapchain_info, allocation_callbacks)? };
        let images = unsafe { device.get_swapchain_images(swapchain)? };
        Ok(Self {
            device,
            swapchain,
            format,
            extent,
            images,
            allocation_callbacks,
        })
    }

    pub fn choose_format(
        available_formats: Vec<vk::SurfaceFormatKHR>,
        preferred_format: vk::Format,
        preferred_color_space: vk::ColorSpaceKHR,
    ) -> vk::SurfaceFormatKHR {
        for format in &available_formats {
            if format.format == preferred_format && format.color_space == preferred_color_space {
                return *format;
            }
        }

        return available_formats[0];
    }

    pub fn choose_extent(
        window: &winit::window::Window,
        capabilities: vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        let window_size = window.inner_size();
        let width = window_size.width;
        let height = window_size.height;

        vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }

    pub fn choose_present_mode(
        available_modes: Vec<vk::PresentModeKHR>,
        preferred_mode: vk::PresentModeKHR,
    ) -> vk::PresentModeKHR {
        for mode in available_modes {
            if mode == preferred_mode {
                return mode;
            }
        }

        vk::PresentModeKHR::FIFO // guaranteed to have
    }
}

impl<'a> Drop for Swapchain<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_swapchain(self.swapchain, self.allocation_callbacks);
        }
    }
}
