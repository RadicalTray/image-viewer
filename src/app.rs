use ash::{khr, vk};
use winit::raw_window_handle::HasWindowHandle;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    raw_window_handle::HasDisplayHandle,
    window::{Window, WindowId},
};

pub struct App {
    vk_entry: ash::Entry,
    vk_instance: Option<ash::Instance>,
    window: Option<Window>,
    surface_instance: Option<khr::surface::Instance>,
    surface: Option<vk::SurfaceKHR>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        assert!(self.vk_instance.is_none());
        assert!(self.window.is_none());
        assert!(self.surface_instance.is_none());
        assert!(self.surface.is_none());

        self.init_vk_instance(event_loop);
        self.init_window(event_loop);
        self.init_surface();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Exiting.");
                self.cleanup();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => (),
        }
    }
}

impl App {
    pub fn new() -> Self {
        App {
            vk_entry: ash::Entry::linked(),
            vk_instance: None,
            window: None,
            surface_instance: None,
            surface: None,
        }
    }

    fn draw(&self) {}

    fn init_vk_instance(&mut self, event_loop: &ActiveEventLoop) {
        let vk_entry = &self.vk_entry;

        let mut enabled_extensions = Vec::from(
            ash_window::enumerate_required_extensions(
                event_loop.display_handle().unwrap().as_raw(),
            )
            .unwrap(),
        );

        enabled_extensions.push(vk::EXT_DEBUG_UTILS_NAME.as_ptr());
        let enabled_layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];

        let app_info = vk::ApplicationInfo::default()
            .application_name(c"Image Viewer")
            .application_version(vk::make_api_version(0, 1, 0, 0));

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layers);

        self.vk_instance = unsafe {
            Some(
                vk_entry
                    .create_instance(&create_info, None)
                    .expect("Failed to create vulkan instance."),
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

        self.surface_instance = Some(khr::surface::Instance::new(vk_entry, vk_instance));

        self.surface = unsafe {
            Some(
                ash_window::create_surface(
                    vk_entry,
                    vk_instance,
                    window.display_handle().unwrap().as_raw(),
                    window.window_handle().unwrap().as_raw(),
                    None,
                )
                .expect("Failed to create surface."),
            )
        };
    }

    // Is implementing this on Drop possible? and is it even a good idea?
    /// used on WindowEvent::CloseRequested
    fn cleanup(&mut self) {
        unsafe {
            self.surface_instance
                .take()
                .unwrap()
                .destroy_surface(self.surface.take().unwrap(), None);
            self.vk_instance.take().unwrap().destroy_instance(None);
        }
    }
}
