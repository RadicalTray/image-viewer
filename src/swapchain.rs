use crate::{device::Device, instance::Instance};
use ash::{khr, prelude::*, vk};
use std::{error::Error, rc::Rc};

pub struct Swapchain<'a> {
    ash_device: Rc<Device<'a>>,
    device: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    framebuffers: Option<Vec<vk::Framebuffer>>,
    allocator: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> Swapchain<'a> {
    pub unsafe fn new(
        ash_instance: &Instance,
        ash_device: Rc<Device<'a>>,
        swapchain_info: &vk::SwapchainCreateInfoKHR,
        allocator: Option<&'a vk::AllocationCallbacks<'a>>,
    ) -> VkResult<Self> {
        let format = swapchain_info.image_format;
        let extent = swapchain_info.image_extent;
        let device = khr::swapchain::Device::new(ash_instance.instance(), ash_device.device());

        unsafe {
            let swapchain = device.create_swapchain(swapchain_info, allocator)?;
            let images = device.get_swapchain_images(swapchain)?;
            let allocation_callbacks = allocator;
            let mut image_views = Vec::with_capacity(images.len());
            for image in &images {
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
                image_views.push(
                    ash_device
                        .device()
                        .create_image_view(&image_view_info, allocation_callbacks)?,
                );
            }
            Ok(Self {
                ash_device,
                device,
                swapchain,
                format,
                extent,
                images,
                allocator,
                image_views,
                framebuffers: None,
            })
        }
    }

    pub unsafe fn init_framebuffers(
        &mut self,
        render_pass: vk::RenderPass,
    ) -> Result<(), Box<dyn Error>> {
        let image_views = &self.image_views;

        let mut framebuffers = Vec::with_capacity(image_views.len());
        for image_view in image_views {
            let attachments = [*image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(self.extent().width)
                .height(self.extent().height)
                .layers(1);

            let framebuffer = unsafe {
                self.ash_device
                    .device()
                    .create_framebuffer(&framebuffer_info, None)
                    .unwrap()
            };

            framebuffers.push(framebuffer);
        }

        self.framebuffers = Some(framebuffers);
        Ok(())
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

    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> VkResult<(u32, bool)> {
        unsafe {
            self.device()
                .acquire_next_image(self.swapchain(), timeout, semaphore, fence)
        }
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

    pub fn framebuffers(&self) -> Option<&Vec<vk::Framebuffer>> {
        self.framebuffers.as_ref()
    }
}

impl<'a> Drop for Swapchain<'a> {
    fn drop(&mut self) {
        let device = self.ash_device.device();
        let allocator = self.allocator;
        unsafe {
            self.image_views
                .iter()
                .for_each(|x| device.destroy_image_view(*x, allocator));
            if let Some(f) = self.framebuffers.take() {
                f.into_iter()
                    .for_each(|x| device.destroy_framebuffer(x, allocator));
            }
            self.device
                .destroy_swapchain(self.swapchain, self.allocator);
        }
    }
}
