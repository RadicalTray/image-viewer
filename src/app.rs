use crate::{
    constants::*, physical_device::PhysicalDevice, queue_family_indices::QueueFamilyIndices,
    shader_module::ShaderModule, swapchain::Swapchain, vertex::Vertex,
};
use ash::{
    ext, khr,
    vk::{
        self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCallbackDataEXT,
    },
};
use std::{
    collections::HashSet,
    ffi::{CStr, c_char, c_void},
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

pub struct App<'a> {
    vk_entry: ash::Entry,
    vk_instance: Option<ash::Instance>,
    window: Option<Window>,
    surface_instance: Option<khr::surface::Instance>,
    surface: Option<vk::SurfaceKHR>,
    debug_messenger_instance: Option<ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    queue_family_indices: Option<QueueFamilyIndices>,
    physical_device: Option<PhysicalDevice>,
    device: Option<ash::Device>,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    swapchain: Option<Swapchain<'a>>,
    swapchain_image_views: Option<Vec<vk::ImageView>>,
    render_pass: Option<vk::RenderPass>,
    descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    graphics_pipeline_layout: Option<vk::PipelineLayout>,
    graphics_pipeline: Option<vk::Pipeline>,
    swapchain_framebuffers: Option<Vec<vk::Framebuffer>>,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}

/// clean up on Drop
impl<'a> App<'a> {
    pub fn new() -> Self {
        App {
            vk_entry: ash::Entry::linked(),
            vk_instance: None,
            window: None,
            surface_instance: None,
            surface: None,
            debug_messenger_instance: None,
            debug_messenger: None,
            queue_family_indices: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            present_queue: None,
            swapchain: None,
            swapchain_image_views: None,
            render_pass: None,
            descriptor_set_layout: None,
            graphics_pipeline_layout: None,
            graphics_pipeline: None,
            swapchain_framebuffers: None,
        }
    }

    fn assert_null(&self) {
        assert!(self.vk_instance.is_none());
        assert!(self.window.is_none());
        assert!(self.surface_instance.is_none());
        assert!(self.surface.is_none());
        assert!(self.debug_messenger_instance.is_none());
        assert!(self.debug_messenger.is_none());
        assert!(self.physical_device.is_none());
        assert!(self.device.is_none());
        assert!(self.graphics_queue.is_none());
        assert!(self.present_queue.is_none());
    }

    fn init(&mut self, event_loop: &ActiveEventLoop) {
        self.assert_null();
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
    }

    fn init_vk_instance(&mut self, event_loop: &ActiveEventLoop) {
        let vk_entry = &self.vk_entry;

        let mut enabled_extension_names = Vec::from(
            ash_window::enumerate_required_extensions(
                event_loop.display_handle().unwrap().as_raw(),
            )
            .unwrap(),
        );

        // TODO: disable this on release build
        enabled_extension_names.extend(DEBUG_ENABLED_EXTENSION_NAMES);
        let enabled_layer_names = Vec::from(DEBUG_ENABLED_LAYER_NAMES);
        let mut debug_info =
            populate_debug_create_info(vk::DebugUtilsMessengerCreateInfoEXT::default());

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

        self.vk_instance = unsafe {
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
            self.vk_entry
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
        let available_layers =
            unsafe { self.vk_entry.enumerate_instance_layer_properties().unwrap() };

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
        let vk_entry = &self.vk_entry;
        let vk_instance = self.vk_instance.as_ref().unwrap();

        let debug_info =
            populate_debug_create_info(vk::DebugUtilsMessengerCreateInfoEXT::default());

        self.debug_messenger_instance =
            Some(ext::debug_utils::Instance::new(vk_entry, &vk_instance));
        let debug_messenger_instance = self.debug_messenger_instance.as_ref().unwrap();

        self.debug_messenger = unsafe {
            Some(
                debug_messenger_instance
                    .create_debug_utils_messenger(&debug_info, None)
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
        let vk_entry = &self.vk_entry;
        let vk_instance = self.vk_instance.as_ref().unwrap();
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
        let vk_instance = self.vk_instance.as_ref().unwrap();
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
        let vk_instance = self.vk_instance.as_ref().unwrap();
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

        let vk_instance = self.vk_instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let swapchain = Swapchain::new(vk_instance, device, &swapchain_info, None).unwrap();
        self.swapchain_image_views = Some(swapchain.get_image_views(device).unwrap());
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
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE) // winit vs glfw?
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
        let swapchain_image_views = self.swapchain_image_views.as_ref().unwrap();
        let swapchain = self.swapchain.as_ref().unwrap();
        let render_pass = *self.render_pass.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();

        let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());
        for image_view in swapchain_image_views {
            let attachments = [*image_view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain.extent().width)
                .height(swapchain.extent().height)
                .layers(1);

            let framebuffer =
                unsafe { device.create_framebuffer(&framebuffer_info, None).unwrap() };

            framebuffers.push(framebuffer);
        }

        self.swapchain_framebuffers = Some(framebuffers);
    }
}

impl<'a> Drop for App<'a> {
    // i should probably use macro lol
    fn drop(&mut self) {
        let vk_instance = self.vk_instance.take().unwrap();
        let device = self.device.take().unwrap();
        let debug_messenger_instance = self.debug_messenger_instance.take().unwrap();
        let surface_instance = self.surface_instance.take().unwrap();
        let swapchain = self.swapchain.take().unwrap();
        let swapchain_image_views = self.swapchain_image_views.take().unwrap();
        let render_pass = self.render_pass.take().unwrap();
        let descriptor_set_layout = self.descriptor_set_layout.take().unwrap();
        let graphics_pipeline_layout = self.graphics_pipeline_layout.take().unwrap();
        let graphics_pipeline = self.graphics_pipeline.take().unwrap();
        let swapchain_framebuffers = self.swapchain_framebuffers.take().unwrap();

        unsafe {
            swapchain_framebuffers
                .into_iter()
                .for_each(|framebuffer| device.destroy_framebuffer(framebuffer, None));
            device.destroy_pipeline_layout(graphics_pipeline_layout, None);
            device.destroy_pipeline(graphics_pipeline, None);
            device.destroy_descriptor_set_layout(descriptor_set_layout, None);
            device.destroy_render_pass(render_pass, None);
            swapchain_image_views
                .into_iter()
                .for_each(|image_view| device.destroy_image_view(image_view, None));
            drop(swapchain);
            surface_instance.destroy_surface(self.surface.take().unwrap(), None);
            debug_messenger_instance
                .destroy_debug_utils_messenger(self.debug_messenger.take().unwrap(), None);
            device.destroy_device(None);
            vk_instance.destroy_instance(None);
        }
    }
}

fn populate_debug_create_info(
    debug_info: vk::DebugUtilsMessengerCreateInfoEXT,
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    debug_info
        .message_severity(
            DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                // | DebugUtilsMessageSeverityFlagsEXT::INFO
                | DebugUtilsMessageSeverityFlagsEXT::WARNING
                | DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            DebugUtilsMessageTypeFlagsEXT::GENERAL
                | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_callback))
}

unsafe extern "system" fn debug_callback(
    _: DebugUtilsMessageSeverityFlagsEXT,
    _: DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const DebugUtilsMessengerCallbackDataEXT<'_>,
    _: *mut c_void,
) -> vk::Bool32 {
    let s = unsafe { CStr::from_ptr((*callback_data).p_message) };
    println!("DEBUG: {}", String::from_utf8_lossy(s.to_bytes()));
    vk::FALSE
}
