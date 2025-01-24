use crate::vertex::Vertex;
use ash::vk;
use glam::{vec2, vec3};
use std::ffi::c_char;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const DEBUG_ENABLED_EXTENSION_NAMES: [*const c_char; 1] = [vk::EXT_DEBUG_UTILS_NAME.as_ptr()];
pub const DEBUG_ENABLED_LAYER_NAMES: [*const c_char; 1] = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
pub const ENABLED_DEVICE_EXTENSION_NAMES: [*const c_char; 1] = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];
pub const VERTICES: [Vertex; 4] = [
    Vertex {
        pos: vec2(-0.5, -0.5),
        color: vec3(1.0, 0.0, 0.0),
    },
    Vertex {
        pos: vec2(0.5, -0.5),
        color: vec3(0.0, 1.0, 0.0),
    },
    Vertex {
        pos: vec2(0.5, 0.5),
        color: vec3(0.0, 0.0, 1.0),
    },
    Vertex {
        pos: vec2(-0.5, 0.5),
        color: vec3(1.0, 1.0, 1.0),
    },
];
pub const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub fn check_physical_device_features(
    physical_device_features: vk::PhysicalDeviceFeatures,
) -> bool {
    let feats = physical_device_features;

    feats.geometry_shader == vk::TRUE && feats.sampler_anisotropy == vk::TRUE
}
