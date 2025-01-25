mod app;
mod buffer;
mod command_pool;
mod constants;
mod debug_messenger;
mod fence;
mod physical_device;
mod pipeline;
mod queue;
mod semaphore;
mod shader_module;
mod surface;
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
