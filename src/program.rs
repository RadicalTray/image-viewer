use ash::{Entry, vk};
use std::error::Error;

pub struct Program {}

impl Program {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::init_window()?;
        Self::init_vulkan()?;
        Ok(Program {})
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn init_window() -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn init_vulkan() -> Result<(), Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(())
    }
}
