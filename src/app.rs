use crate::{
    buffer::Buffer,
    constants::*,
    debug_messenger::{self, DebugMessenger},
    physical_device::PhysicalDevice,
    queue_family_indices::QueueFamilyIndices,
    shader_module::ShaderModule,
    swapchain::Swapchain,
    uniform_buffer_object::UniformBufferObject,
    vertex::Vertex,
};
use ash::{khr, vk};
use core::f32;
use glam::{Mat4, vec3};
use std::{
    collections::HashSet,
    ffi::{CStr, c_char},
    fs,
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
    ash_entry: ash::Entry,
    ash_instance: Option<ash::Instance>,
    window: Option<Window>,
    surface_instance: Option<khr::surface::Instance>,
    surface: Option<vk::SurfaceKHR>,
    debug_messenger: Option<DebugMessenger>,
    queue_family_indices: Option<QueueFamilyIndices>,
    physical_device: Option<PhysicalDevice>,
    device: Option<ash::Device>,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    swapchain: Option<Swapchain>,
    render_pass: Option<vk::RenderPass>,
    descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    graphics_pipeline_layout: Option<vk::PipelineLayout>,
    graphics_pipeline: Option<vk::Pipeline>,
    command_pool: Option<vk::CommandPool>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    uniform_buffers: Option<Vec<Buffer>>,
    descriptor_pool: Option<vk::DescriptorPool>,
    descriptor_sets: Option<Vec<vk::DescriptorSet>>,
    command_buffers: Option<Vec<vk::CommandBuffer>>,
    image_available_sems: Option<Vec<vk::Semaphore>>,
    render_finished_sems: Option<Vec<vk::Semaphore>>,
    in_flight_fences: Option<Vec<vk::Fence>>,
    current_frame: usize,
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
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(_) => {
                self.recreate_swapchain();
            }
            _ => (),
        }
    }
}

/// clean up on Drop
impl App {
    pub fn new(vk_entry: ash::Entry) -> Self {
        App {
            ash_entry: vk_entry,
            ash_instance: None,
            window: None,
            surface_instance: None,
            surface: None,
            debug_messenger: None,
            queue_family_indices: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            present_queue: None,
            swapchain: None,
            render_pass: None,
            descriptor_set_layout: None,
            graphics_pipeline_layout: None,
            graphics_pipeline: None,
            command_pool: None,
            vertex_buffer: None,
            index_buffer: None,
            uniform_buffers: None,
            descriptor_pool: None,
            descriptor_sets: None,
            command_buffers: None,
            image_available_sems: None,
            render_finished_sems: None,
            in_flight_fences: None,
            current_frame: 0,
            start_time: std::time::SystemTime::now(),
        }
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) {
        self.init_vk_instance(event_loop);
        self.init_debug_messenger();
        self.init_window(event_loop);
        self.init_surface();
        self.init_physical_device();
        self.init_logical_device();
        self.init_swapchain();
        self.init_render_pass();
        self.init_descriptor_set_layout();
        self.init_graphics_pipeline();
        self.init_framebuffers();
        self.init_command_pool();
        self.init_vertex_buffer();
        self.init_index_buffer();
        self.init_uniform_buffers();
        self.init_descriptor_pool();
        self.init_descriptor_sets();
        self.init_command_buffers();
        self.init_sync_objects();
    }

    fn init_vk_instance(&mut self, event_loop: &ActiveEventLoop) {
        let vk_entry = &self.ash_entry;

        let mut enabled_extension_names = Vec::from(
            ash_window::enumerate_required_extensions(
                event_loop.display_handle().unwrap().as_raw(),
            )
            .unwrap(),
        );

        // TODO: disable this on release build
        enabled_extension_names.extend(DEBUG_ENABLED_EXTENSION_NAMES);
        let enabled_layer_names = Vec::from(DEBUG_ENABLED_LAYER_NAMES);
        let mut debug_info = debug_messenger::populate_debug_create_info(
            vk::DebugUtilsMessengerCreateInfoEXT::default(),
        );

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

        self.ash_instance = unsafe {
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
            self.ash_entry
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
        let available_layers = unsafe {
            self.ash_entry
                .enumerate_instance_layer_properties()
                .unwrap()
        };

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
        let ash_entry = &self.ash_entry;
        let ash_instance = self.ash_instance.as_ref().unwrap();

        let create_info = debug_messenger::populate_debug_create_info(
            vk::DebugUtilsMessengerCreateInfoEXT::default(),
        );

        self.debug_messenger = unsafe {
            Some(
                DebugMessenger::new(ash_entry, ash_instance, &create_info)
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
        let vk_entry = &self.ash_entry;
        let vk_instance = self.ash_instance.as_ref().unwrap();
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

    fn init_physical_device(&mut self) {
        let vk_instance = self.ash_instance.as_ref().unwrap();
        let physical_devices = unsafe {
            vk_instance
                .enumerate_physical_devices()
                .expect("Unable to enumerate physical devices.")
        };

        let mut chosen_device = None;
        let mut chosen_queue_family_indices = None;
        for device in physical_devices {
            let device = PhysicalDevice::new(device);
            let queue_family_properties = device.query_queue_family_properties(&vk_instance);

            let surface_instance = self.surface_instance.as_ref().unwrap();
            let surface = self.surface.as_ref().unwrap();

            let mut queue_family_indices = QueueFamilyIndices::default();
            for (i, property) in queue_family_properties.iter().enumerate() {
                let support_surface = device
                    .query_support_surface(surface_instance, i.try_into().unwrap(), *surface)
                    .unwrap();

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

            let supported_features = device.query_features(vk_instance);

            if !(device
                .support_extensions(vk_instance, &ENABLED_DEVICE_EXTENSION_NAMES)
                .unwrap()
                && queue_family_indices.is_complete()
                && check_physical_device_features(supported_features))
            {
                continue;
            }

            let supported_surface_format = device
                .query_supported_surface_formats(surface_instance, *surface)
                .unwrap();
            let supported_present_modes = device
                .query_supported_present_modes(surface_instance, *surface)
                .unwrap();

            if supported_surface_format.is_empty() || supported_present_modes.is_empty() {
                continue;
            }

            chosen_device = Some(device);
            chosen_queue_family_indices = Some(queue_family_indices);
        }

        if chosen_device.is_none() || chosen_queue_family_indices.is_none() {
            panic!("Failed to find suitable physical device");
        }

        self.physical_device = chosen_device;
        self.queue_family_indices = chosen_queue_family_indices;
    }

    fn init_logical_device(&mut self) {
        let vk_instance = self.ash_instance.as_ref().unwrap();
        let physical_device = self.physical_device.as_ref().unwrap();
        let queue_family_indices = self.queue_family_indices.as_ref().unwrap();
        let present_family = queue_family_indices.present_family.unwrap();
        let graphics_family = queue_family_indices.graphics_family.unwrap();

        let unique_indices = HashSet::from([present_family, graphics_family]);

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

        let device = physical_device
            .create_logical_device(vk_instance, &device_info, None)
            .unwrap();

        self.graphics_queue = unsafe { Some(device.get_device_queue(graphics_family, 0)) };
        self.present_queue = unsafe { Some(device.get_device_queue(present_family, 0)) };
        self.device = Some(device);
    }

    fn init_swapchain(&mut self) {
        let physical_device = self.physical_device.as_ref().unwrap();
        let surface_instance = self.surface_instance.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();

        let format = Swapchain::choose_format(
            physical_device
                .query_supported_surface_formats(surface_instance, *surface)
                .unwrap(),
            vk::Format::B8G8R8A8_SRGB,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        );

        let capabilities = physical_device
            .query_surface_capabilities(surface_instance, *surface)
            .unwrap();
        let swapchain_extent =
            Swapchain::choose_extent(self.window.as_ref().unwrap(), capabilities);

        let present_mode = Swapchain::choose_present_mode(
            physical_device
                .query_supported_present_modes(surface_instance, *surface)
                .unwrap(),
            vk::PresentModeKHR::FIFO, // prefer this for power saving
        );

        let max_image_count = capabilities.max_image_count;
        let pref_image_count = capabilities.min_image_count + 1;

        let mut image_count = pref_image_count;
        if max_image_count != 0 && pref_image_count > max_image_count {
            image_count = max_image_count;
        }

        let mut swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(swapchain_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        let queue_family_indices = self.queue_family_indices.as_ref().unwrap();
        let graphics_family_idx = queue_family_indices.graphics_family.unwrap();
        let present_family_idx = queue_family_indices.present_family.unwrap();

        let indices = [graphics_family_idx, present_family_idx];
        if graphics_family_idx != present_family_idx {
            swapchain_info = swapchain_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&indices);
        } else {
            swapchain_info = swapchain_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        swapchain_info = swapchain_info
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let vk_instance = self.ash_instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let swapchain =
            unsafe { Swapchain::new(vk_instance, device, &swapchain_info, None).unwrap() };
        self.swapchain = Some(swapchain);
    }

    fn init_render_pass(&mut self) {
        let device = self.device.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();

        let color_attachment = vk::AttachmentDescription::default()
            .format(swapchain.format())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachments = [color_attachment_ref];

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments);

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::NONE)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];
        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None).unwrap() };

        self.render_pass = Some(render_pass);
    }

    fn init_descriptor_set_layout(&mut self) {
        let device = self.device.as_ref().unwrap();

        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let bindings = [ubo_layout_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .unwrap()
        };

        self.descriptor_set_layout = Some(descriptor_set_layout);
    }

    fn init_graphics_pipeline(&mut self) {
        let device = self.device.as_ref().unwrap();

        let vert_shader_code = fs::read("build/shaders/vert.spv").unwrap();
        let frag_shader_code = fs::read("build/shaders/frag.spv").unwrap();

        let vert_shader_module = ShaderModule::new(device, &vert_shader_code, None).unwrap();
        let frag_shader_module = ShaderModule::new(device, &frag_shader_code, None).unwrap();

        let vert_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module.module())
            .name(c"main");

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module.module())
            .name(c"main");

        let shader_stage_infos = [vert_shader_stage_info, frag_shader_stage_info];
        let bind_desc = Vertex::get_binding_descriptions();
        let attr_desc = Vertex::get_attribute_descriptions();

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&bind_desc)
            .vertex_attribute_descriptions(&attr_desc);

        let input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0f32);

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            );

        let color_blend_attachments = [color_blend_attachment_state];
        let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let descriptor_set_layouts = [*self.descriptor_set_layout.as_ref().unwrap()];
        let pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_set_layouts);

        let graphics_pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .unwrap()
        };

        let render_pass = *self.render_pass.as_ref().unwrap();
        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stage_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_state_info)
            .multisample_state(&multisample_state_info)
            .color_blend_state(&color_blend_state_info)
            .dynamic_state(&dynamic_state_info)
            .layout(graphics_pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let create_infos = [graphics_pipeline_info];
        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None)
                .unwrap()
        };

        self.graphics_pipeline_layout = Some(graphics_pipeline_layout);
        self.graphics_pipeline = Some(graphics_pipeline[0]);
    }

    fn init_framebuffers(&mut self) {
        let device = self.device.as_ref().unwrap();
        let render_pass = self.render_pass.as_ref().unwrap();

        unsafe {
            self.swapchain
                .as_mut()
                .unwrap()
                .init_framebuffers(device, *render_pass)
                .unwrap();
        }
    }

    fn init_command_pool(&mut self) {
        let indices = self.queue_family_indices.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();

        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.graphics_family.unwrap());

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .unwrap()
        };

        self.command_pool = Some(command_pool);
    }

    fn init_vertex_buffer(&mut self) {
        let vk_instance = self.ash_instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let physical_device = self.physical_device.as_ref().unwrap();
        let device_mem_props = physical_device.query_memory_properties(vk_instance);

        let buffer_size: vk::DeviceSize = size_of_val(&VERTICES).try_into().unwrap();
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let mut staging_buffer = Buffer::new(
            device,
            &buffer_info,
            None,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_mem_props,
        )
        .unwrap();

        unsafe {
            staging_buffer
                .map_memory(device, 0, vk::MemoryMapFlags::empty())
                .unwrap();
            staging_buffer
                .ptr()
                .unwrap()
                .copy_from(VERTICES.as_ptr().cast(), buffer_size.try_into().unwrap());
            staging_buffer.unmap_memory(device);
        };

        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = Buffer::new(
            device,
            &buffer_info,
            None,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_mem_props,
        )
        .unwrap();

        self.copy_buffer_into(staging_buffer.buffer(), vertex_buffer.buffer(), buffer_size);

        staging_buffer.cleanup(device, None);

        self.vertex_buffer = Some(vertex_buffer);
    }

    fn copy_buffer_into(
        &self,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) {
        let command_pool = *self.command_pool.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&command_buffer_info)
                .unwrap()[0]
        };

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap()
        };

        let copy_region = vk::BufferCopy::default().size(size);
        let regions = [copy_region];
        unsafe { device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &regions) };

        unsafe { device.end_command_buffer(command_buffer).unwrap() };

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);
        let submits = [submit_info];
        let graphics_queue = *self.graphics_queue.as_ref().unwrap();
        unsafe {
            device
                .queue_submit(graphics_queue, &submits, vk::Fence::null())
                .unwrap();
            device.queue_wait_idle(graphics_queue).unwrap();
            device.free_command_buffers(command_pool, &command_buffers);
        };
    }

    fn init_index_buffer(&mut self) {
        let vk_instance = self.ash_instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let physical_device = self.physical_device.as_ref().unwrap();
        let device_mem_props = physical_device.query_memory_properties(vk_instance);

        let buffer_size: vk::DeviceSize = size_of_val(&INDICES).try_into().unwrap();
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let mut staging_buffer = Buffer::new(
            device,
            &buffer_info,
            None,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_mem_props,
        )
        .unwrap();

        unsafe {
            staging_buffer
                .map_memory(device, 0, vk::MemoryMapFlags::empty())
                .unwrap();
            staging_buffer
                .ptr()
                .unwrap()
                .copy_from(INDICES.as_ptr().cast(), buffer_size.try_into().unwrap());
            staging_buffer.unmap_memory(device);
        };

        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let index_buffer = Buffer::new(
            device,
            &buffer_info,
            None,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_mem_props,
        )
        .unwrap();

        self.copy_buffer_into(staging_buffer.buffer(), index_buffer.buffer(), buffer_size);

        staging_buffer.cleanup(device, None);

        self.index_buffer = Some(index_buffer);
    }

    fn init_uniform_buffers(&mut self) {
        let vk_instance = self.ash_instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let physical_device = self.physical_device.as_ref().unwrap();
        let device_mem_props = physical_device.query_memory_properties(vk_instance);

        let buffer_size: vk::DeviceSize = size_of::<UniformBufferObject>().try_into().unwrap();
        let mut uniform_buffers = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::BufferCreateInfo::default()
                .size(buffer_size)
                .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let mut buffer = Buffer::new(
                device,
                &buffer_info,
                None,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                device_mem_props,
            )
            .unwrap();

            buffer
                .map_memory(device, 0, vk::MemoryMapFlags::empty())
                .unwrap();

            uniform_buffers.push(buffer);
        }

        self.uniform_buffers = Some(uniform_buffers);
    }

    fn init_descriptor_pool(&mut self) {
        let device = self.device.as_ref().unwrap();

        let pool_size = vk::DescriptorPoolSize::default()
            .descriptor_count(MAX_FRAMES_IN_FLIGHT.try_into().unwrap());
        let pool_sizes = [pool_size];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(MAX_FRAMES_IN_FLIGHT.try_into().unwrap())
            .pool_sizes(&pool_sizes);

        let pool = unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() };
        self.descriptor_pool = Some(pool);
    }

    fn init_descriptor_sets(&mut self) {
        let layout = *self.descriptor_set_layout.as_ref().unwrap();
        let descriptor_pool = *self.descriptor_pool.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let uniform_buffers = self.uniform_buffers.as_ref().unwrap();

        let layouts = vec![layout; MAX_FRAMES_IN_FLIGHT.try_into().unwrap()];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap() };

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(uniform_buffers[i].buffer())
                .offset(0)
                .range(size_of::<UniformBufferObject>().try_into().unwrap());
            let buffer_infos = [buffer_info];
            let desc_write = vk::WriteDescriptorSet::default()
                .dst_set(sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos);
            unsafe { device.update_descriptor_sets(&[desc_write], &[]) };
        }
        self.descriptor_sets = Some(sets);
    }

    fn init_command_buffers(&mut self) {
        let device = self.device.as_ref().unwrap();
        let command_pool = *self.command_pool.as_ref().unwrap();

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT.try_into().unwrap());

        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info).unwrap() };
        self.command_buffers = Some(command_buffers);
    }

    fn init_sync_objects(&mut self) {
        let device = self.device.as_ref().unwrap();

        let mut image_available_sems = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_sems = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_sems.push(device.create_semaphore(&sem_info, None).unwrap());
                render_finished_sems.push(device.create_semaphore(&sem_info, None).unwrap());
                in_flight_fences.push(device.create_fence(&fence_info, None).unwrap());
            }
        }

        self.image_available_sems = Some(image_available_sems);
        self.render_finished_sems = Some(render_finished_sems);
        self.in_flight_fences = Some(in_flight_fences);
    }

    fn draw(&mut self) {
        let device = self.device.as_ref().unwrap();
        let in_flight_fences = self.in_flight_fences.as_ref().unwrap();
        let command_buffers = self.command_buffers.as_ref().unwrap();
        let current_frame = self.current_frame;
        let swapchain = self.swapchain.as_ref().unwrap();

        let command_buffer = command_buffers[current_frame];

        unsafe {
            device
                .wait_for_fences(&[in_flight_fences[current_frame]], true, u64::MAX)
                .unwrap();

            let image_available_sems = self.image_available_sems.as_ref().unwrap();
            let (image_index, _is_suboptimal) = match swapchain.acquire_next_image(
                u64::MAX,
                image_available_sems[current_frame],
                vk::Fence::null(),
            ) {
                Ok(t) => t,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain();
                    return;
                }
                _ => panic!(),
            };

            device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .unwrap();

            self.record_command_buffer(command_buffer, image_index);
            self.update_uniform_buffers(current_frame);

            let swapchain = self.swapchain.as_ref().unwrap();
            let device = self.device.as_ref().unwrap();
            let image_available_sems = self.image_available_sems.as_ref().unwrap();
            let render_finished_sems = self.render_finished_sems.as_ref().unwrap();
            let command_buffers = self.command_buffers.as_ref().unwrap();
            let in_flight_fences = self.in_flight_fences.as_ref().unwrap();

            let wait_sems = [image_available_sems[current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [command_buffers[current_frame]];
            let signal_sems = [render_finished_sems[current_frame]];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_sems)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_sems);

            device
                .reset_fences(&[in_flight_fences[current_frame]])
                .unwrap();

            device
                .queue_submit(
                    *self.graphics_queue.as_ref().unwrap(),
                    &[submit_info],
                    in_flight_fences[current_frame],
                )
                .unwrap();

            let swapchains = [swapchain.swapchain()];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_sems)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            match swapchain
                .device()
                .queue_present(*self.present_queue.as_ref().unwrap(), &present_info)
            {
                Ok(is_suboptimal) => {
                    if is_suboptimal {
                        self.recreate_swapchain()
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.recreate_swapchain(),
                _ => panic!(),
            }
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn record_command_buffer(&mut self, command_buffer: vk::CommandBuffer, image_index: u32) {
        let device = self.device.as_ref().unwrap();
        let render_pass = *self.render_pass.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let graphics_pipeline = *self.graphics_pipeline.as_ref().unwrap();
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();
        let index_buffer = self.index_buffer.as_ref().unwrap();
        let pipeline_layout = *self.graphics_pipeline_layout.as_ref().unwrap();
        let descriptor_sets = self.descriptor_sets.as_ref().unwrap();
        let current_frame = self.current_frame;

        let image_index: usize = image_index.try_into().unwrap();
        let framebuffers = swapchain.framebuffers().unwrap();
        let framebuffer = framebuffers[image_index];

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
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
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
                pipeline_layout,
                0,
                &[descriptor_sets[current_frame]],
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

    fn update_uniform_buffers(&mut self, current_frame: usize) {
        let swapchain = self.swapchain.as_ref().unwrap();
        let start_time = self.start_time;
        let uniform_buffers = self.uniform_buffers.as_ref().unwrap();

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
            uniform_buffers[current_frame]
                .ptr()
                .unwrap()
                .copy_from(&raw const ubo as *const _, size_of_val(&ubo))
        };
    }

    fn recreate_swapchain(&mut self) {
        let swapchain = self.swapchain.take().unwrap();
        let device = self.device.as_ref().unwrap();

        unsafe {
            device.device_wait_idle().unwrap();
            swapchain.cleanup(device, None);
        };

        self.init_swapchain();
        self.init_framebuffers();
    }
}

impl Drop for App {
    // i should probably use macro lol
    fn drop(&mut self) {
        let device = self.device.take().unwrap();
        let surface_instance = self.surface_instance.take().unwrap();
        let render_pass = self.render_pass.take().unwrap();
        let descriptor_set_layout = self.descriptor_set_layout.take().unwrap();
        let graphics_pipeline_layout = self.graphics_pipeline_layout.take().unwrap();
        let graphics_pipeline = self.graphics_pipeline.take().unwrap();
        let command_pool = self.command_pool.take().unwrap();
        let vertex_buffer = self.vertex_buffer.take().unwrap();
        let index_buffer = self.index_buffer.take().unwrap();
        let uniform_buffers = self.uniform_buffers.take().unwrap();
        let descriptor_pool = self.descriptor_pool.take().unwrap();
        let image_available_sems = self.image_available_sems.take().unwrap();
        let render_finished_sems = self.render_finished_sems.take().unwrap();
        let in_flight_fences = self.in_flight_fences.take().unwrap();
        let sems_chain = image_available_sems
            .into_iter()
            .chain(render_finished_sems.into_iter());

        unsafe {
            device.device_wait_idle().unwrap();

            sems_chain.for_each(|x| device.destroy_semaphore(x, None));
            in_flight_fences
                .into_iter()
                .for_each(|x| device.destroy_fence(x, None));
            device.destroy_descriptor_pool(descriptor_pool, None);
            uniform_buffers
                .into_iter()
                .for_each(|x| x.cleanup(&device, None));
            index_buffer.cleanup(&device, None);
            vertex_buffer.cleanup(&device, None);
            device.destroy_command_pool(command_pool, None);
            device.destroy_pipeline_layout(graphics_pipeline_layout, None);
            device.destroy_pipeline(graphics_pipeline, None);
            device.destroy_descriptor_set_layout(descriptor_set_layout, None);
            device.destroy_render_pass(render_pass, None);
            self.swapchain.take().unwrap().cleanup(&device, None);
            surface_instance.destroy_surface(self.surface.take().unwrap(), None);
            self.debug_messenger.take().unwrap().cleanup(None);
            device.destroy_device(None);
            self.ash_instance.take().unwrap().destroy_instance(None);
        }
    }
}
