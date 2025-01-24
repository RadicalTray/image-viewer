use crate::instance::Instance;
use ash::{khr, prelude::*, vk};
use std::rc::Rc;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct Surface<'a> {
    _ash_instance: Rc<Instance>,
    instance: khr::surface::Instance,
    surface: vk::SurfaceKHR,
    allocator: Option<&'a vk::AllocationCallbacks<'a>>,
}

impl<'a> Surface<'a> {
    pub unsafe fn new(
        ash_entry: &ash::Entry,
        ash_instance: Rc<Instance>,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        allocator: Option<&'a vk::AllocationCallbacks<'a>>,
    ) -> VkResult<Self> {
        let instance = khr::surface::Instance::new(ash_entry, ash_instance.instance());
        let surface = unsafe {
            ash_window::create_surface(
                ash_entry,
                ash_instance.instance(),
                display_handle,
                window_handle,
                allocator,
            )
        }?;

        Ok(Self {
            _ash_instance: ash_instance,
            instance,
            surface,
            allocator,
        })
    }

    pub fn instance(&self) -> &khr::surface::Instance {
        &self.instance
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }
}

impl<'a> Drop for Surface<'a> {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_surface(self.surface, self.allocator);
        }
    }
}
