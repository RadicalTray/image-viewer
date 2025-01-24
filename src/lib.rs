mod app;
mod buffer;
mod constants;
mod physical_device;
mod queue_family_indices;
mod shader_module;
mod swapchain;
mod uniform_buffer_object;
mod vertex;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run() {
    let event_loop = EventLoop::new().expect("Failed to create event loop.");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(unsafe { ash::Entry::load().unwrap() });
    event_loop.run_app(&mut app).unwrap();
}
