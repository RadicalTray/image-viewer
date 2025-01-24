#[repr(C, align(16))]
#[derive(Debug)]
pub struct UniformBufferObject {
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
}
