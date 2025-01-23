use crate::vertex::Vertex;
use ash::vk;
use std::ffi::c_char;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;
pub const DEBUG_ENABLED_EXTENSION_NAMES: [*const c_char; 1] = [vk::EXT_DEBUG_UTILS_NAME.as_ptr()];
pub const DEBUG_ENABLED_LAYER_NAMES: [*const c_char; 1] = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
pub const ENABLED_DEVICE_EXTENSION_NAMES: [*const c_char; 1] = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];
pub const VERTICES: [Vertex; 4] = [
    Vertex {
        pos: glam::Vec2 { x: -0.5, y: 0.5 },
        color: glam::Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
    },
    Vertex {
        pos: glam::Vec2 { x: -0.5, y: 0.5 },
        color: glam::Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    },
    Vertex {
        pos: glam::Vec2 { x: -0.5, y: 0.5 },
        color: glam::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    },
    Vertex {
        pos: glam::Vec2 { x: -0.5, y: 0.5 },
        color: glam::Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    },
];
pub const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub fn check_physical_device_features(
    physical_device_features: vk::PhysicalDeviceFeatures,
) -> bool {
    let feats = physical_device_features;

    feats.geometry_shader == vk::TRUE && feats.sampler_anisotropy == vk::TRUE
}
