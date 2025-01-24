use crate::instance::Instance;
use ash::{
    ext,
    prelude::*,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT,
    },
};
use std::{
    ffi::{CStr, c_void},
    rc::Rc,
};

pub struct DebugMessenger<'a> {
    _ash_instance: Rc<Instance>,
    instance: ext::debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
    allocator: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> DebugMessenger<'a> {
    pub unsafe fn new(
        vk_entry: &ash::Entry,
        ash_instance: Rc<Instance>,
        create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
        allocator: Option<&'a vk::AllocationCallbacks<'a>>,
    ) -> VkResult<Self> {
        let instance = ext::debug_utils::Instance::new(vk_entry, ash_instance.instance());

        let messenger = unsafe { instance.create_debug_utils_messenger(create_info, allocator)? };

        Ok(Self {
            _ash_instance: ash_instance,
            instance,
            messenger,
            allocator,
        })
    }
}

impl<'a> Drop for DebugMessenger<'a> {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .destroy_debug_utils_messenger(self.messenger, self.allocator);
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
