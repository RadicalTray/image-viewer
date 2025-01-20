use crate::{
    constants::*, physical_device::PhysicalDevice, queue_family_indices::QueueFamilyIndices,
};
use ash::{
    ext, khr,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT,
    },
};
use std::{
    collections::HashSet,
    ffi::{CStr, c_char, c_void},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    raw_window_handle::HasDisplayHandle,
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowId},
};

pub struct App {
    vk_entry: ash::Entry,
    vk_instance: Option<ash::Instance>,
    window: Option<Window>,
    surface_instance: Option<khr::surface::Instance>,
    surface: Option<vk::SurfaceKHR>,
    debug_messenger_instance: Option<ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    physical_device: Option<PhysicalDevice>,
    device: Option<ash::Device>,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => (),
        }
    }
}

/// clean up on Drop
impl App {
    pub fn new() -> Self {
        App {
            vk_entry: ash::Entry::linked(),
            vk_instance: None,
            window: None,
            surface_instance: None,
            surface: None,
            debug_messenger_instance: None,
            debug_messenger: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            present_queue: None,
        }
    }

    fn assert_null(&self) {
        assert!(self.vk_instance.is_none());
        assert!(self.window.is_none());
        assert!(self.surface_instance.is_none());
        assert!(self.surface.is_none());
        assert!(self.debug_messenger_instance.is_none());
        assert!(self.debug_messenger.is_none());
        assert!(self.physical_device.is_none());
        assert!(self.device.is_none());
        assert!(self.graphics_queue.is_none());
        assert!(self.present_queue.is_none());
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) {
        self.assert_null();
        self.init_vk_instance(event_loop);
        self.init_debug_messenger();
        self.init_window(event_loop);
        self.init_surface();
        self.init_physical_device();
        self.init_logical_device();
    }

    fn init_vk_instance(&mut self, event_loop: &ActiveEventLoop) {
        let vk_entry = &self.vk_entry;

        let mut enabled_extension_names = Vec::from(
            ash_window::enumerate_required_extensions(
                event_loop.display_handle().unwrap().as_raw(),
            )
            .unwrap(),
        );

        // TODO: disable this on release build
        enabled_extension_names.extend(DEBUG_ENABLED_EXTENSION_NAMES);
        let enabled_layer_names = Vec::from(DEBUG_ENABLED_LAYER_NAMES);
        let mut debug_info =
            populate_debug_create_info(vk::DebugUtilsMessengerCreateInfoEXT::default());

        let enabled_extension_names = self.check_extensions_support(enabled_extension_names);
        let enabled_layer_names = self.check_layers_support(enabled_layer_names);
        println!("Enabled extensions:");
        for name in &enabled_extension_names {
            let x_cstr = unsafe { CStr::from_ptr(*name) };
            println!("\t{}", String::from_utf8_lossy(x_cstr.to_bytes()));
        }
        println!("Enabled layers:");
        for name in &enabled_layer_names {
            let x_cstr = unsafe { CStr::from_ptr(*name) };
            println!("\t{}", String::from_utf8_lossy(x_cstr.to_bytes()));
        }

        let app_info = vk::ApplicationInfo::default()
            .application_name(c"Image Viewer")
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names)
            .enabled_layer_names(&enabled_layer_names)
            .push_next(&mut debug_info);

        self.vk_instance = unsafe {
            Some(
                vk_entry
                    .create_instance(&create_info, None)
                    .expect("Failed to create vulkan instance."),
            )
        };
    }

    fn check_extensions_support(
        &self,
        mut enabled_extension_names: Vec<*const c_char>,
    ) -> Vec<*const c_char> {
        let available_extensions = unsafe {
            self.vk_entry
                .enumerate_instance_extension_properties(None)
                .unwrap()
        };

        enabled_extension_names.retain(|x| {
            let x_cstr = unsafe { CStr::from_ptr(*x) };
            if available_extensions
                .iter()
                .any(|y| x_cstr == y.extension_name_as_c_str().unwrap())
            {
                true
            } else {
                println!(
                    "Extension {} is not supported!",
                    String::from_utf8_lossy(x_cstr.to_bytes())
                );
                false
            }
        });
        enabled_extension_names
    }

    fn check_layers_support(
        &self,
        mut enabled_layer_names: Vec<*const c_char>,
    ) -> Vec<*const c_char> {
        let available_layers =
            unsafe { self.vk_entry.enumerate_instance_layer_properties().unwrap() };

        enabled_layer_names.retain(|x| {
            let x_cstr = unsafe { CStr::from_ptr(*x) };
            if available_layers
                .iter()
                .any(|y| x_cstr == y.layer_name_as_c_str().unwrap())
            {
                true
            } else {
                println!(
                    "Layer {} is not supported!",
                    String::from_utf8_lossy(x_cstr.to_bytes())
                );
                false
            }
        });

        enabled_layer_names
    }

    fn init_debug_messenger(&mut self) {
        let vk_entry = &self.vk_entry;
        let vk_instance = self.vk_instance.as_ref().unwrap();

        let debug_info =
            populate_debug_create_info(vk::DebugUtilsMessengerCreateInfoEXT::default());

        self.debug_messenger_instance =
            Some(ext::debug_utils::Instance::new(vk_entry, &vk_instance));
        let debug_messenger_instance = self.debug_messenger_instance.as_ref().unwrap();

        self.debug_messenger = unsafe {
            Some(
                debug_messenger_instance
                    .create_debug_utils_messenger(&debug_info, None)
                    .expect("Failed to create debug messenger."),
            )
        };
    }

    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        self.window = Some(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to create window."),
        );
    }

    fn init_surface(&mut self) {
        let vk_entry = &self.vk_entry;
        let vk_instance = self.vk_instance.as_ref().unwrap();
        let window = self.window.as_ref().unwrap();

        self.surface_instance = Some(khr::surface::Instance::new(vk_entry, &vk_instance));

        self.surface = unsafe {
            Some(
                ash_window::create_surface(
                    vk_entry,
                    &vk_instance,
                    window.display_handle().unwrap().as_raw(),
                    window.window_handle().unwrap().as_raw(),
                    None,
                )
                .expect("Failed to create surface."),
            )
        };
    }

    // TODO:
    //  [ ] check device extension support
    //  [ ] abstract device
    fn init_physical_device(&mut self) {
        let vk_instance = self.vk_instance.as_ref().unwrap();
        let physical_devices = unsafe {
            vk_instance
                .enumerate_physical_devices()
                .expect("Unable to enumerate physical devices.")
        };
        for device in physical_devices {
            self.physical_device = Some(PhysicalDevice::new(device));
        }
    }

    fn init_logical_device(&mut self) {
        let vk_instance = self.vk_instance.as_ref().unwrap();
        let physical_device = self.physical_device.as_ref().unwrap();
        let surface_instance = self.surface_instance.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();

        let queue_family_properties =
            unsafe { physical_device.get_queue_family_properties(&vk_instance) };

        let mut queue_family_indices = QueueFamilyIndices::default();
        for (i, property) in queue_family_properties.iter().enumerate() {
            let support_surface = unsafe {
                physical_device
                    .support_surface(surface_instance, i.try_into().unwrap(), *surface)
                    .unwrap()
            };

            if support_surface {
                queue_family_indices.present_family = Some(i.try_into().unwrap());
            }

            if property.queue_flags.intersects(vk::QueueFlags::GRAPHICS) {
                queue_family_indices.graphics_family = Some(i.try_into().unwrap());
            }

            if queue_family_indices.is_complete() {
                break;
            }
        }

        let unique_indices = HashSet::from([
            queue_family_indices.present_family.unwrap(),
            queue_family_indices.graphics_family.unwrap(),
        ]);

        let mut queue_create_infos = Vec::with_capacity(unique_indices.len());
        let queue_priority = [1.0f32];
        for idx in unique_indices {
            let queue_create_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(idx)
                .queue_priorities(&queue_priority);
            queue_create_infos.push(queue_create_info);
        }

        let features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&features)
            .enabled_extension_names(&ENABLED_DEVICE_EXTENSION_NAMES);

        let device = unsafe {
            physical_device
                .create_logical_device(vk_instance, &device_info, None)
                .unwrap()
        };

        self.graphics_queue = unsafe {
            Some(device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0))
        };
        self.present_queue = unsafe {
            Some(device.get_device_queue(queue_family_indices.present_family.unwrap(), 0))
        };
        self.device = Some(device);
    }

    fn draw(&self) {}
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            drop(self.physical_device.take());
            self.surface_instance
                .take()
                .unwrap()
                .destroy_surface(self.surface.take().unwrap(), None);
            self.vk_instance.take().unwrap().destroy_instance(None);
            self.debug_messenger_instance
                .take()
                .unwrap()
                .destroy_debug_utils_messenger(self.debug_messenger.take().unwrap(), None);
        }
    }
}

fn populate_debug_create_info(
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
