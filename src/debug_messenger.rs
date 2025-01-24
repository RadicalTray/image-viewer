use std::ffi::{CStr, c_void};

use ash::{
    ext,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT,
    },
};

pub struct DebugMessenger<'a> {
    vk_instance: &'a ash::Instance,
    instance: ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl<'a> DebugMessenger<'a> {
    pub fn new(vk_entry: &ash::Entry, vk_instance: &'a ash::Instance) -> Self {
        let debug_info =
            populate_debug_create_info(vk::DebugUtilsMessengerCreateInfoEXT::default());

        let instance = ext::debug_utils::Instance::new(vk_entry, &vk_instance);

        let messenger = unsafe {
            instance
                .create_debug_utils_messenger(&debug_info, None)
                .expect("Failed to create debug messenger.")
        };

        Self {
            vk_instance,
            instance,
            messenger,
        }
    }
}

pub fn populate_debug_create_info(
    debug_info: vk::DebugUtilsMessengerCreateInfoEXT,
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    debug_info
        .message_severity(
            DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                // | DebugUtilsMessageSeverityFlagsEXT::INFO
                | DebugUtilsMessageSeverityFlagsEXT::WARNING
                | DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            DebugUtilsMessageTypeFlagsEXT::GENERAL
                | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_callback))
}

unsafe extern "system" fn debug_callback(
    _: DebugUtilsMessageSeverityFlagsEXT,
    _: DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const DebugUtilsMessengerCallbackDataEXT<'_>,
    _: *mut c_void,
) -> vk::Bool32 {
    let s = unsafe { CStr::from_ptr((*callback_data).p_message) };
    println!("DEBUG: {}", String::from_utf8_lossy(s.to_bytes()));
    vk::FALSE
}
