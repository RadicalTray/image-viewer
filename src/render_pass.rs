use ash::prelude::*;
use ash::vk;

pub struct RenderPass {
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub unsafe fn new(
        ash_device: &ash::Device,
        create_info: &vk::RenderPassCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let render_pass = unsafe { ash_device.create_render_pass(create_info, allocator)? };
        Ok(Self { render_pass })
    }

    pub unsafe fn cleanup(
        self,
        ash_device: &ash::Device,
        allocator: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            ash_device.destroy_render_pass(self.render_pass, allocator);
        }
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }
}
