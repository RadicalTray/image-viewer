use ash::prelude::*;
use ash::vk;

pub struct DescriptorSetLayout {
    layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub unsafe fn new(
        ash_device: &ash::Device,
        create_info: &vk::DescriptorSetLayoutCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let layout = unsafe { ash_device.create_descriptor_set_layout(create_info, allocator)? };
        Ok(Self { layout })
    }

    pub unsafe fn cleanup(
        self,
        ash_device: &ash::Device,
        allocator: Option<&vk::AllocationCallbacks>,
    ) {
        unsafe {
            ash_device.destroy_descriptor_set_layout(self.layout, allocator);
        }
    }

    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
}
