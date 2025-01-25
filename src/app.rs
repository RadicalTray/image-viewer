use crate::{constants::*, engine::Engine, uniform_buffer_object::UniformBufferObject};
use ash::vk;
use core::f32;
use glam::{Mat4, vec3};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

pub struct App {
    engine: Engine,
    start_time: std::time::SystemTime,
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
                self.engine.window().request_redraw();
            }
            WindowEvent::Resized(_) => {
                self.engine.recreate_swapchain();
            }
            _ => (),
        }
    }
}

impl App {
    pub fn new(ash_entry: ash::Entry) -> Self {
        Self {
            engine: Engine::new(ash_entry),
            start_time: std::time::SystemTime::now(),
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) {
        self.engine.init(event_loop);
    }

    fn draw(&mut self) {
        let engine = &self.engine;
        let device = engine.device();
        let in_flight_fence = engine.in_flight_fence();
        let swapchain = engine.swapchain();
        let command_buffer = engine.command_buffer();

        unsafe {
            device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)
                .unwrap();

            let image_available_sem = engine.image_available_sem();
            let (image_index, _is_suboptimal) = match swapchain.acquire_next_image(
                u64::MAX,
                image_available_sem,
                vk::Fence::null(),
            ) {
                Ok(t) => t,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.engine.recreate_swapchain();
                    return;
                }
                _ => panic!(),
            };

            device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();

            self.record_command_buffer(command_buffer, image_index.try_into().unwrap());
            self.update_uniform_buffers();

            let engine = &self.engine;
            let swapchain = engine.swapchain();
            let device = engine.device();
            let image_available_sem = engine.image_available_sem();
            let render_finished_sem = engine.render_finished_sem();
            let command_buffer = engine.command_buffer();

            let wait_sems = [image_available_sem];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [command_buffer];
            let signal_sems = [render_finished_sem];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_sems)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_sems);

            device.reset_fences(&[in_flight_fence]).unwrap();

            device
                .queue_submit(engine.graphics_queue(), &[submit_info], in_flight_fence)
                .unwrap();

            let swapchains = [swapchain.swapchain()];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_sems)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            match swapchain
                .device()
                .queue_present(engine.present_queue(), &present_info)
            {
                Ok(is_suboptimal) => {
                    if is_suboptimal {
                        self.engine.recreate_swapchain()
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.engine.recreate_swapchain(),
                _ => panic!(),
            }
        }

        self.engine.next_frame();
    }

    fn record_command_buffer(&mut self, command_buffer: vk::CommandBuffer, image_index: usize) {
        let engine = &self.engine;
        let device = engine.device();
        let render_pass = engine.render_pass();
        let swapchain = engine.swapchain();
        let pipeline = engine.graphics_pipeline();
        let vertex_buffer = engine.vertex_buffer();
        let index_buffer = engine.index_buffer();

        let layout = pipeline.layout();
        let pipeline = pipeline.pipeline();

        let framebuffer = engine.framebuffer(image_index);

        unsafe {
            let begin_info = vk::CommandBufferBeginInfo::default();
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();

            let clear_values = [{
                let mut clear_color = vk::ClearValue::default();
                clear_color.color.float32 = [0.0, 0.0, 0.0, 1.0];
                clear_color
            }];
            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D::default().x(0).y(0),
                    extent: swapchain.extent(),
                })
                .clear_values(&clear_values);

            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.buffer()], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                index_buffer.buffer(),
                0,
                vk::IndexType::UINT32,
            );
            device.cmd_set_viewport(command_buffer, 0, &[vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(swapchain.extent().width as f32)
                .height(swapchain.extent().height as f32)
                .min_depth(0.0)
                .max_depth(1.0)]);
            device.cmd_set_scissor(command_buffer, 0, &[vk::Rect2D::default()
                .offset(vk::Offset2D::default().x(0).y(0))
                .extent(swapchain.extent())]);

            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                &[engine.descriptor_set()],
                &[],
            );

            device.cmd_draw_indexed(
                command_buffer,
                INDICES.len().try_into().unwrap(),
                1,
                0,
                0,
                0,
            );
            device.cmd_end_render_pass(command_buffer);
            device.end_command_buffer(command_buffer).unwrap();
        }
    }

    fn update_uniform_buffers(&mut self) {
        let engine = &self.engine;
        let swapchain = engine.swapchain();
        let start_time = self.start_time;

        let time_elapsed = start_time.elapsed().unwrap().as_secs_f32();
        let pi = f32::consts::PI;
        let aspect_ratio: f32 = swapchain.extent().width as f32 / swapchain.extent().height as f32;

        let ubo = UniformBufferObject {
            model: Mat4::from_rotation_z(time_elapsed * pi / 2.0),
            view: Mat4::look_at_rh(
                vec3(2.0, 2.0, 2.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 0.0, 1.0),
            ),
            proj: Mat4::perspective_rh(-pi / 4.0, aspect_ratio, 0.1, 10.0),
        };

        unsafe {
            engine
                .uniform_buffer_ptr()
                .copy_from(&raw const ubo as *const _, size_of_val(&ubo))
        };
    }
}
