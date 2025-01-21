use std::error::Error;

use ash::{khr, prelude::*, vk};

// lifetime is 100% not working properly, but everything is working because we're using no allocation_callbacks
pub struct Swapchain<'a> {
    device: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Option<Vec<vk::ImageView>>,
    framebuffers: Option<Vec<vk::Framebuffer>>,
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
            image_views: None,
            framebuffers: None,
        })
    }

    pub fn init_image_views(&mut self, vk_device: &ash::Device) -> VkResult<()> {
        let images = &self.images;
        let format = self.format;
        let allocation_callbacks = self.allocation_callbacks;
        let mut image_views = Vec::with_capacity(images.len());
        for image in images {
            let swizzle_identity = vk::ComponentSwizzle::IDENTITY;
            let image_view_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(
                    vk::ComponentMapping::default()
                        .r(swizzle_identity)
                        .g(swizzle_identity)
                        .b(swizzle_identity)
                        .a(swizzle_identity),
                )
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );
            image_views.push(unsafe {
                vk_device.create_image_view(&image_view_info, allocation_callbacks)?
            });
        }

        self.image_views = Some(image_views);
        Ok(())
    }

    pub fn init_framebuffers(
        &mut self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
    ) -> Result<(), Box<dyn Error>> {
        let swapchain_image_views = self.image_views.as_ref().unwrap();

        let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());
        for image_view in swapchain_image_views {
            let attachments = [*image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(self.extent().width)
                .height(self.extent().height)
                .layers(1);

            let framebuffer =
                unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() };

            framebuffers.push(framebuffer);
        }

        self.framebuffers = Some(framebuffers);
        Ok(())
    }

    pub fn cleanup(
        mut self,
        device: &ash::Device,
        allocation_callbacks: Option<&'a vk::AllocationCallbacks<'_>>,
    ) {
        unsafe {
            self.framebuffers
                .take()
                .unwrap()
                .into_iter()
                .for_each(|x| device.destroy_framebuffer(x, allocation_callbacks));
            self.image_views
                .take()
                .unwrap()
                .into_iter()
                .for_each(|x| device.destroy_image_view(x, allocation_callbacks));
        }
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

    pub fn device(&self) -> &khr::swapchain::Device {
        &self.device
    }

    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn images(&self) -> &Vec<vk::Image> {
        &self.images
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
