use ash::{khr, prelude::*, vk};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct Surface {
    instance: khr::surface::Instance,
    surface: vk::SurfaceKHR,
}

impl Surface {
    pub unsafe fn new(
        ash_entry: &ash::Entry,
        ash_instance: &ash::Instance,
        window: &Window,
    ) -> VkResult<Self> {
        let instance = khr::surface::Instance::new(ash_entry, ash_instance);
        let surface = unsafe {
            ash_window::create_surface(
                ash_entry,
                ash_instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )?
        };
        Ok(Self { instance, surface })
    }

    pub unsafe fn cleanup(self, allocator: Option<&vk::AllocationCallbacks>) {
        unsafe {
            self.instance.destroy_surface(self.surface, allocator);
        }
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn instance(&self) -> &khr::surface::Instance {
        &self.instance
    }
}
