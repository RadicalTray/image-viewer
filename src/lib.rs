mod app;
mod constants;
mod physical_device;
mod queue_family_indices;
mod swapchain;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn run() {
    let event_loop = EventLoop::new().expect("Failed to create event loop.");
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
