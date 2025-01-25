use crate::{
    buffer::Buffer,
    command_pool::CommandPool,
    constants::*,
    debug_messenger::{self, DebugMessenger},
    descriptor_pool::DescriptorPool,
    descriptor_set_layout::DescriptorSetLayout,
    device::Device,
    fence::Fence,
    instance::Instance,
    physical_device::PhysicalDevice,
    pipeline::Pipeline,
    queue::{QueueFamilyIndices, Queues},
    render_pass::RenderPass,
    semaphore::Semaphore,
    shader_module::ShaderModule,
    surface::Surface,
    swapchain::Swapchain,
    uniform_buffer_object::UniformBufferObject,
    vertex::Vertex,
};
use ash::vk;
use std::{
    collections::HashSet,
    ffi::{CStr, c_char, c_void},
    fs,
};
use winit::{event_loop::ActiveEventLoop, raw_window_handle::HasDisplayHandle, window::Window};

pub struct Engine {
    ash_entry: ash::Entry,
    ash_instance: Option<Instance>,
    window: Option<Window>,
    surface: Option<Surface>,
    debug_messenger: Option<DebugMessenger>,
    queue_family_indices: Option<QueueFamilyIndices>,
    physical_device: Option<PhysicalDevice>,
    device: Option<Device>,
    queues: Option<Queues>,
    swapchain: Option<Swapchain>,
    render_pass: Option<RenderPass>,
    descriptor_set_layout: Option<DescriptorSetLayout>,
    graphics_pipeline: Option<Pipeline>,
    command_pool: Option<CommandPool>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    uniform_buffers: Option<Vec<Buffer>>,
    descriptor_pool: Option<DescriptorPool>,
    descriptor_sets: Option<Vec<vk::DescriptorSet>>,
    command_buffers: Option<Vec<vk::CommandBuffer>>,
    image_available_sems: Option<Vec<Semaphore>>,
    render_finished_sems: Option<Vec<Semaphore>>,
    in_flight_fences: Option<Vec<Fence>>,
    current_frame: usize,
}

/// clean up on Drop
impl Engine {
    pub fn new(ash_entry: ash::Entry) -> Self {
        Self {
            ash_entry,
            ash_instance: None,
            window: None,
            surface: None,
            debug_messenger: None,
            queue_family_indices: None,
            physical_device: None,
            device: None,
            queues: None,
            swapchain: None,
            render_pass: None,
            descriptor_set_layout: None,
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
        }
    }

    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        self.init_ash_instance(event_loop);
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

    fn init_ash_instance(&mut self, event_loop: &ActiveEventLoop) {
        let ash_entry = &self.ash_entry;

        let mut enabled_extension_names = Vec::from(
            ash_window::enumerate_required_extensions(
                event_loop.display_handle().unwrap().as_raw(),
            )
            .unwrap(),
        );
        let mut enabled_layer_names = Vec::new();

        if cfg!(debug_assertions) {
            enabled_extension_names.extend(DEBUG_ENABLED_EXTENSION_NAMES);
            enabled_layer_names.extend(DEBUG_ENABLED_LAYER_NAMES);
        }

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
            .enabled_layer_names(&enabled_layer_names);

        let mut debug_info = debug_messenger::populate_debug_create_info(
            vk::DebugUtilsMessengerCreateInfoEXT::default(),
        );

        let create_info = if cfg!(debug_assertions) {
            create_info.push_next(&mut debug_info)
        } else {
            create_info
        };

        self.ash_instance = unsafe {
            Some(
                Instance::new(ash_entry, &create_info, None)
                    .expect("Failed to create vulkan instance."),
            )
        };

        if cfg!(debug_assertions) {
            self.init_debug_messenger();
        }
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
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();

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
        let ash_entry = &self.ash_entry;
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
        let window = self.window.as_ref().unwrap();

        self.surface = unsafe {
            Some(Surface::new(ash_entry, ash_instance, window).expect("Failed to create surface."))
        };
    }

    fn init_physical_device(&mut self) {
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
        let physical_devices = unsafe {
            ash_instance
                .enumerate_physical_devices()
                .expect("Unable to enumerate physical devices.")
        };

        let mut chosen_device = None;
        let mut chosen_queue_family_indices = None;
        for device in physical_devices {
            let device = PhysicalDevice::from(device);
            let queue_family_properties = device.query_queue_family_properties(&ash_instance);

            let surface = self.surface.as_ref().unwrap();
            let surface_instance = surface.instance();
            let surface = surface.surface();

            let mut queue_family_indices = QueueFamilyIndices::default();
            for (i, property) in queue_family_properties.iter().enumerate() {
                let support_surface = device
                    .query_support_surface(surface_instance, i.try_into().unwrap(), surface)
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

            let supported_features = device.query_features(ash_instance);

            if !(device
                .support_extensions(ash_instance, &ENABLED_DEVICE_EXTENSION_NAMES)
                .unwrap()
                && queue_family_indices.is_complete()
                && check_physical_device_features(supported_features))
            {
                continue;
            }

            let supported_surface_format = device
                .query_supported_surface_formats(surface_instance, surface)
                .unwrap();
            let supported_present_modes = device
                .query_supported_present_modes(surface_instance, surface)
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
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
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
            .create_logical_device(ash_instance, &device_info, None)
            .unwrap();

        self.queues = unsafe {
            Some(Queues {
                graphics: device.device().get_device_queue(graphics_family, 0),
                present: device.device().get_device_queue(present_family, 0),
            })
        };
        self.device = Some(device);
    }

    fn init_swapchain(&mut self) {
        let physical_device = self.physical_device.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();
        let surface_instance = surface.instance();
        let surface = surface.surface();

        let format = Swapchain::choose_format(
            physical_device
                .query_supported_surface_formats(surface_instance, surface)
                .unwrap(),
            vk::Format::B8G8R8A8_SRGB,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        );

        let capabilities = physical_device
            .query_surface_capabilities(surface_instance, surface)
            .unwrap();
        let swapchain_extent =
            Swapchain::choose_extent(self.window.as_ref().unwrap(), capabilities);

        let present_mode = Swapchain::choose_present_mode(
            physical_device
                .query_supported_present_modes(surface_instance, surface)
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
            .surface(surface)
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

        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
        let device = self.device.as_ref().unwrap();
        let swapchain = unsafe {
            Swapchain::new(ash_instance, device.device(), &swapchain_info, None).unwrap()
        };
        self.swapchain = Some(swapchain);
    }

    fn init_render_pass(&mut self) {
        let device = self.device.as_ref().unwrap().device();
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

        let render_pass = unsafe { RenderPass::new(device, &render_pass_info, None).unwrap() };

        self.render_pass = Some(render_pass);
    }

    fn init_descriptor_set_layout(&mut self) {
        let device = self.device.as_ref().unwrap().device();

        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let bindings = [ubo_layout_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout =
            unsafe { DescriptorSetLayout::new(device, &layout_info, None).unwrap() };

        self.descriptor_set_layout = Some(descriptor_set_layout);
    }

    fn init_graphics_pipeline(&mut self) {
        let device = self.device.as_ref().unwrap().device();

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
        let bind_desc = Vertex::get_binding_descriptions().unwrap();
        let attr_desc = Vertex::get_attribute_descriptions().unwrap();

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

        let descriptor_set_layouts = [self.descriptor_set_layout.as_ref().unwrap().layout()];
        let pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&descriptor_set_layouts);

        let graphics_pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .unwrap()
        };

        let render_pass = self.render_pass.as_ref().unwrap();
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
            .render_pass(render_pass.render_pass())
            .subpass(0);

        let create_infos = [graphics_pipeline_info];
        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None)
                .unwrap()
        };

        self.graphics_pipeline = Some(Pipeline::from(
            graphics_pipeline_layout,
            graphics_pipeline[0],
        ));
    }

    fn init_framebuffers(&mut self) {
        let device = self.device.as_ref().unwrap().device();
        let render_pass = self.render_pass.as_ref().unwrap();

        unsafe {
            self.swapchain
                .as_mut()
                .unwrap()
                .init_framebuffers(device, render_pass.render_pass())
                .unwrap();
        }
    }

    fn init_command_pool(&mut self) {
        let indices = self.queue_family_indices.as_ref().unwrap();
        let device = self.device.as_ref().unwrap().device();

        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.graphics_family.unwrap());

        let command_pool = unsafe { CommandPool::new(device, &command_pool_info, None).unwrap() };

        self.command_pool = Some(command_pool);
    }

    fn init_vertex_buffer(&mut self) {
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
        let device = self.device.as_ref().unwrap().device();
        let physical_device = self.physical_device.as_ref().unwrap();
        let device_mem_props = physical_device.query_memory_properties(ash_instance);

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
        let command_pool = self.command_pool.as_ref().unwrap();
        let device = self.device.as_ref().unwrap().device();

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool.pool())
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
        let graphics_queue = self.queues.as_ref().unwrap().graphics;
        unsafe {
            device
                .queue_submit(graphics_queue, &submits, vk::Fence::null())
                .unwrap();
            device.queue_wait_idle(graphics_queue).unwrap();
            device.free_command_buffers(command_pool.pool(), &command_buffers);
        };
    }

    fn init_index_buffer(&mut self) {
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
        let device = self.device.as_ref().unwrap().device();
        let physical_device = self.physical_device.as_ref().unwrap();
        let device_mem_props = physical_device.query_memory_properties(ash_instance);

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
        let ash_instance = self.ash_instance.as_ref().unwrap().instance();
        let device = self.device.as_ref().unwrap().device();
        let physical_device = self.physical_device.as_ref().unwrap();
        let device_mem_props = physical_device.query_memory_properties(ash_instance);

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
        let device = self.device.as_ref().unwrap().device();

        let pool_size = vk::DescriptorPoolSize::default()
            .descriptor_count(MAX_FRAMES_IN_FLIGHT.try_into().unwrap());
        let pool_sizes = [pool_size];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(MAX_FRAMES_IN_FLIGHT.try_into().unwrap())
            .pool_sizes(&pool_sizes);

        let pool = unsafe { DescriptorPool::new(device, &pool_info, None).unwrap() };
        self.descriptor_pool = Some(pool);
    }

    fn init_descriptor_sets(&mut self) {
        let layout = self.descriptor_set_layout.as_ref().unwrap().layout();
        let descriptor_pool = self.descriptor_pool.as_ref().unwrap().pool();
        let device = self.device.as_ref().unwrap().device();
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
        let device = self.device.as_ref().unwrap().device();
        let command_pool = self.command_pool.as_ref().unwrap();

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool.pool())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT.try_into().unwrap());

        let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info).unwrap() };
        self.command_buffers = Some(command_buffers);
    }

    fn init_sync_objects(&mut self) {
        let device = self.device.as_ref().unwrap().device();

        let mut image_available_sems = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_sems = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available_sems.push(Semaphore::new(device, &sem_info, None).unwrap());
                render_finished_sems.push(Semaphore::new(device, &sem_info, None).unwrap());
                in_flight_fences.push(Fence::new(device, &fence_info, None).unwrap());
            }
        }

        self.image_available_sems = Some(image_available_sems);
        self.render_finished_sems = Some(render_finished_sems);
        self.in_flight_fences = Some(in_flight_fences);
    }

    pub fn recreate_swapchain(&mut self) {
        let swapchain = self.swapchain.take().unwrap();
        let device = self.device.as_ref().unwrap().device();

        unsafe {
            device.device_wait_idle().unwrap();
            swapchain.cleanup(device, None);
        };

        self.init_swapchain();
        self.init_framebuffers();
    }
}

impl Engine {
    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn in_flight_fence(&self) -> vk::Fence {
        self.in_flight_fences.as_ref().unwrap()[self.current_frame].fence()
    }

    pub fn image_available_sem(&self) -> vk::Semaphore {
        self.image_available_sems.as_ref().unwrap()[self.current_frame].sem()
    }

    pub fn render_finished_sem(&self) -> vk::Semaphore {
        self.render_finished_sems.as_ref().unwrap()[self.current_frame].sem()
    }

    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers.as_ref().unwrap()[self.current_frame]
    }

    pub fn uniform_buffer_ptr(&self) -> *mut c_void {
        self.uniform_buffers.as_ref().unwrap()[self.current_frame]
            .ptr()
            .unwrap()
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_sets.as_ref().unwrap()[self.current_frame]
    }
}

impl Engine {
    pub fn window(&self) -> &Window {
        self.window.as_ref().unwrap()
    }

    pub fn device(&self) -> &ash::Device {
        self.device.as_ref().unwrap().device()
    }

    pub fn swapchain(&self) -> &Swapchain {
        self.swapchain.as_ref().unwrap()
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.queues.as_ref().unwrap().graphics
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.queues.as_ref().unwrap().present
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass.as_ref().unwrap().render_pass()
    }

    pub fn graphics_pipeline(&self) -> &Pipeline {
        self.graphics_pipeline.as_ref().unwrap()
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        self.vertex_buffer.as_ref().unwrap()
    }

    pub fn index_buffer(&self) -> &Buffer {
        self.index_buffer.as_ref().unwrap()
    }

    pub fn framebuffer(&self, image_index: usize) -> vk::Framebuffer {
        self.swapchain
            .as_ref()
            .unwrap()
            .framebuffers()
            .as_ref()
            .unwrap()[image_index]
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let device = self.device.as_ref().unwrap().device();

        unsafe {
            device.device_wait_idle().unwrap();

            self.image_available_sems
                .take()
                .unwrap()
                .into_iter()
                .chain(self.render_finished_sems.take().unwrap().into_iter())
                .for_each(|x| x.cleanup(device, None));
            self.in_flight_fences
                .take()
                .unwrap()
                .into_iter()
                .for_each(|x| x.cleanup(device, None));
            self.descriptor_pool.take().unwrap().cleanup(device, None);
            self.uniform_buffers
                .take()
                .unwrap()
                .into_iter()
                .for_each(|x| x.cleanup(device, None));
            self.index_buffer.take().unwrap().cleanup(device, None);
            self.vertex_buffer.take().unwrap().cleanup(device, None);
            self.command_pool.take().unwrap().cleanup(device, None);
            self.descriptor_set_layout
                .take()
                .unwrap()
                .cleanup(device, None);
            self.render_pass.take().unwrap().cleanup(device, None);
            self.graphics_pipeline.take().unwrap().cleanup(device, None);
            self.swapchain.take().unwrap().cleanup(device, None);
            self.surface.take().unwrap().cleanup(None);
            if cfg!(debug_assertions) {
                self.debug_messenger.take().unwrap().cleanup(None);
            }
            self.device.take().unwrap().cleanup(None);
            self.ash_instance.take().unwrap().cleanup(None);
        }
    }
}
