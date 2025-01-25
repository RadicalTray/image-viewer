use ash::{
    ext,
    prelude::*,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT,
    },
};
use std::ffi::{CStr, c_void};

pub struct DebugMessenger {
    instance: ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub unsafe fn new(
        ash_entry: &ash::Entry,
        ash_instance: &ash::Instance,
        create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
    ) -> VkResult<Self> {
        let instance = ext::debug_utils::Instance::new(ash_entry, ash_instance);

        let messenger = unsafe { instance.create_debug_utils_messenger(create_info, None)? };

        Ok(Self {
            instance,
            messenger,
        })
    }

    pub unsafe fn cleanup(self, allocator: Option<&vk::AllocationCallbacks>) {
        unsafe {
            self.instance
                .destroy_debug_utils_messenger(self.messenger, allocator);
        }
    }
}

pub fn populate_debug_create_info(
    debug_info: vk::DebugUtilsMessengerCreateInfoEXT,
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    debug_info
        .message_severity(
            DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | DebugUtilsMessageSeverityFlagsEXT::INFO
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
