use ash::vk;

pub struct Pipeline {
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn from(layout: vk::PipelineLayout, pipeline: vk::Pipeline) -> Self {
        Self { layout, pipeline }
    }

    pub unsafe fn cleanup(self, device: &ash::Device, allocator: Option<&vk::AllocationCallbacks>) {
        unsafe {
            device.destroy_pipeline(self.pipeline, allocator);
            device.destroy_pipeline_layout(self.layout, allocator);
        }
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }
}
