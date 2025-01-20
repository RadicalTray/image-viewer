use ash::{khr, vk};
use std::ffi::c_char;

pub const DEBUG_ENABLED_EXTENSION_NAMES: [*const c_char; 1] = [vk::EXT_DEBUG_UTILS_NAME.as_ptr()];
pub const DEBUG_ENABLED_LAYER_NAMES: [*const c_char; 1] = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
pub const ENABLED_DEVICE_EXTENSION_NAMES: [*const c_char; 1] = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];
