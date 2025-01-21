use ash::prelude::*;
use ash::vk;

pub struct ShaderModule<'a> {
    device: &'a ash::Device,
    module: vk::ShaderModule,
    allocation_callbacks: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> ShaderModule<'a> {
    pub fn new(
        device: &'a ash::Device,
        code: &Vec<u8>,
        allocation_callbacks: Option<&'a vk::AllocationCallbacks>,
    ) -> VkResult<Self> {
        let mut module_info = vk::ShaderModuleCreateInfo::default();
        module_info.code_size = code.len();
        module_info.p_code = code.as_ptr() as *const u32;

        let module = unsafe { device.create_shader_module(&module_info, allocation_callbacks)? };
        Ok(Self {
            device,
            module,
            allocation_callbacks,
        })
    }

    pub fn module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl<'a> Drop for ShaderModule<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_shader_module(self.module, self.allocation_callbacks)
        };
    }
}
