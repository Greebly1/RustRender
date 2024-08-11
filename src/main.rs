use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, device::{
        physical::PhysicalDevice, 
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, format::Format, image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType}, Image, ImageUsage}, instance::{Instance, InstanceCreateInfo}, library::VulkanLibrary, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, shader::ShaderModule, swapchain::{self, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{self, future::FenceSignalFuture, GpuFuture}, Validated, VulkanError 
};
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy}, window::{Window, WindowAttributes}};
use std::{str::FromStr, sync::Arc};

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 1.0, 1.0, 1.0);
            }
        ",
    }
}

static WINDOW_INIT_WIDTH : u32 = 600;
static WINDOW_INIT_HEIGHT : u32 = 400;

fn main() {
    //1 Connect to GPU
    
    let event_loop: EventLoop<UserEvent> = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let proxy_loop: winit::event_loop::EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let vulkan_driver = initialize_vulkano(&event_loop);

    let mut render_app: Application = Application {
        vk_driver: vulkan_driver,
        event_loop_proxy: Some(proxy_loop),
        window_create_info: Window::default_attributes()
            .with_blur(false)
            .with_inner_size(LogicalSize::new(WINDOW_INIT_WIDTH, WINDOW_INIT_HEIGHT))
            .with_resizable(true)
            .with_title(String::from_str("Test Renderer").unwrap())
            .with_maximized(false)
            .with_visible(true)
            .with_transparent(false),
        ..Default::default()
    };

    event_loop.run_app(&mut render_app).unwrap();


    //2 create window


    //3 create render pass
        //render pass
        //frame buffers

    //4 create vertex buffer


    //5 create pipeline
        //compile shaders
        //subpasses

    //6 create command buffers

    //7 swapchain, make the render pass with the swapchain image
    //flush commands buffer, synchronize, present the swapchain
}

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    Render
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vert {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2]
}

struct Application {
    vk_driver : Arc<Instance>,
    vk_gpu : Option<(Arc<PhysicalDevice>, Vec<u32>)>,
    vk_virtual_gpu : Option<(Arc<Device>, Vec<Arc<Queue>>)>,

    swapchain_invalid : bool,
    event_loop_proxy : Option<EventLoopProxy<UserEvent>>,
    window_create_info : WindowAttributes,
    window_main : Option<(Arc<Window>, Arc<Surface>)>,
    swapchain : Option<(Arc<Swapchain>, Vec<Arc<Image>>)>,

    memory_allocator : Option<Arc<dyn MemoryAllocator>>,
    command_allocator : Option<StandardCommandBufferAllocator>,
    vert_buffer : Option<Subbuffer<[Vert]>>,
    render_pass : Option<Arc<RenderPass>>,
    render_buffers : Option<Vec<Arc<Framebuffer>>>,

    vert_shader : Option<Arc<ShaderModule>>,
    frag_shader : Option<Arc<ShaderModule>>,

    render_pipeline : Option<Arc<GraphicsPipeline>>,
    command_buffers : Option<Vec<Arc<PrimaryAutoCommandBuffer>>>
}

impl Default for Application {
    fn default() -> Self {
        Self { 
            vk_driver: {vulkano::instance::Instance::new(vulkano::VulkanLibrary::new().unwrap(), InstanceCreateInfo::default()).unwrap()}, 
            vk_gpu: None,
            vk_virtual_gpu: None, 
            event_loop_proxy: None,
            window_create_info: WindowAttributes::default(), 
            window_main: None, 
            swapchain: None, 
            memory_allocator: None, 
            command_allocator: None, 
            vert_buffer: None, 
            render_pass: None, 
            render_buffers: None, 
            vert_shader: None, 
            frag_shader: None, 
            render_pipeline: None,
            command_buffers: None,
            swapchain_invalid: false }
    }
}

impl Application {
    fn build_window(&mut self, event_loop : &ActiveEventLoop) {
        let new_window = Arc::new(event_loop.create_window(self.window_create_info.clone())
        .unwrap());

        //https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
        //Says that making a surface is platform specific, so we might need to do conditional compilation for certain platforms
        let window_surface = Surface::from_window(self.vk_driver.clone(), new_window.clone())
                .unwrap();

        self.window_main = Some((new_window, window_surface))
    }

    fn initialize_gpu(&mut self) {
        let device_extensions = DeviceExtensions{
            khr_swapchain: true,
            ..Default::default()
        };
        let surface = self.window_main.as_ref().unwrap().1.clone();

        let device = locate_device(self.vk_driver.clone(), &device_extensions, surface).unwrap();
        self.vk_gpu = Some(device);
    }

    fn initialize_virtual_gpu(&mut self) {
        let (gpu, queue_indices) = self.vk_gpu.as_ref().unwrap().clone();
        let virtual_gpu_params : DeviceCreateInfo;
        
        let vk_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let device_features = Features {
            ..Default::default()
        };

        let queue_params = QueueCreateInfo {
            queue_family_index: queue_indices[0],
            ..Default::default()
        };

        virtual_gpu_params = DeviceCreateInfo {
            enabled_extensions: vk_extensions,
            enabled_features: device_features,
            queue_create_infos: vec![queue_params],
            ..Default::default()
        };

        let (device, queues_iter) = Device::new(gpu, virtual_gpu_params).unwrap();

        let queues: Vec<Arc<Queue>> = queues_iter.collect();

        self.vk_virtual_gpu = Some((device, queues))
    }

    fn build_swapchain(&mut self) {
        let (gpu, queues) = self.vk_virtual_gpu.as_ref().unwrap();
        let (window, surface) = self.window_main.as_ref().unwrap();
        
        let capabilities = self.vk_virtual_gpu.as_ref().unwrap().0
            .physical_device()
            .surface_capabilities(surface, Default::default())
            .unwrap();
        let window_size = self.window_extend();
        let composite_alpha = capabilities.supported_composite_alpha.into_iter().next().unwrap();
        let format = self.vk_gpu.as_ref().unwrap().0
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;

        let swapchain_parameters = SwapchainCreateInfo {
            min_image_count: capabilities.min_image_count + 1,
            image_format: format,
            image_extent: window_size,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: composite_alpha,
            present_mode : PresentMode::Mailbox,
            ..Default::default()
        };

        let (swapchain, swapchain_images) = Swapchain::new(gpu.clone(), surface.clone(), swapchain_parameters).unwrap();
        self.swapchain = Some((swapchain, swapchain_images));
    }

    fn build_renderpass(&mut self) {
        let render_pass : Arc<RenderPass> = vulkano::single_pass_renderpass!(
            self.vk_virtual_gpu.as_ref().unwrap().0.clone(),
            attachments: {
                color: {
                    format: self.render_format_or(Format::R8G8B8A8_UNORM),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ).unwrap();

        self.render_pass = Some(render_pass);
    }

    fn build_framebuffers(&mut self) {
        assert!(self.swapchain.is_some());
        assert!(self.render_pass.is_some());

        let render_pass: Arc<RenderPass> = self.render_pass.as_ref().unwrap().clone();
        let swapchain: (Arc<Swapchain>, Vec<Arc<Image>>) = self.swapchain.as_ref().unwrap().clone();
        let window_size = self.window_main.as_ref().unwrap().0.inner_size();

        let render_buffers : Vec<Arc<Framebuffer>> = swapchain.1.into_iter().map(|img| {
            let img_format = img.format();
            let img_view = ImageView::new(
                img.clone(),
                ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    format: img_format,
                    usage: ImageUsage::COLOR_ATTACHMENT,
                    subresource_range: img.subresource_range(),
                    ..Default::default()
                },
            ).unwrap();

            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![img_view],
                    extent: [window_size.width, window_size.height],
                    ..Default::default()
                }
            ).unwrap()
        }).collect();

        self.render_buffers = Some(render_buffers);
    }

    fn compile_shaders(&mut self) {
        let gpu = self.vk_virtual_gpu.as_ref().unwrap().0.clone();
        self.vert_shader = Some(vs::load(gpu.clone()).unwrap());
        self.frag_shader = Some(fs::load(gpu.clone()).unwrap());
    }

    fn build_pipeline(&mut self) {
        let window_size = self.window_main.as_ref().unwrap().0.inner_size();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [f32::from_bits(window_size.width), f32::from_bits(window_size.height)],
            depth_range: 0.0..=1.0, //<-- since the format is UNORM I think
        };
        
        let vs = self.vert_shader.as_ref().unwrap().clone().entry_point("main").unwrap();
        let fs = self.frag_shader.as_ref().unwrap().clone().entry_point("main").unwrap();

        let vert_input = Vert::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs)
        ];

        let vk_device = self.vk_virtual_gpu.as_ref().unwrap().0.clone();

        let layout = PipelineLayout::new(
            vk_device.clone(), 
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages).into_pipeline_layout_create_info(vk_device.clone()).unwrap()
        ).unwrap();

        let subpass = Subpass::from(self.render_pass.as_ref().unwrap().clone(), 0).unwrap();

        let graphic_pipeline = GraphicsPipeline::new(
            vk_device.clone(), 
            None, 
            GraphicsPipelineCreateInfo{
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vert_input), 
                input_assembly_state: Some(InputAssemblyState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        ).unwrap();

        self.render_pipeline = Some(graphic_pipeline);
    }

    fn build_command_buffers(&mut self) {
        let framebuffers = self.render_buffers.as_ref().unwrap().clone();
        let command_allocator = self.command_allocator.as_ref().unwrap().clone();
        let queue_index = self.vk_virtual_gpu.as_ref().unwrap().1.clone()[0].queue_family_index();
        let pipeline = self.render_pipeline.as_ref().unwrap().clone();
        let vert_buffer = self.vert_buffer.as_ref().unwrap().clone();

        let new_command_buffers : Vec<Arc<PrimaryAutoCommandBuffer>> = framebuffers
            .iter()
            .map(|framebuffer| { 
                let mut builder = AutoCommandBufferBuilder::primary(
                    command_allocator, 
                    queue_index, 
                    CommandBufferUsage::MultipleSubmit).unwrap();

                builder 
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        }, 
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        }
                    ).unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, vert_buffer.clone())
                    .unwrap()
                    .draw(vert_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();

                    builder.build().unwrap()
            })
            .collect();

        self.command_buffers = Some(new_command_buffers);
    }

    fn rebuild_swapchain(&mut self) {
        //remakes the swapchain from the previous one with up to date window size
        //then remakes all the dependent objects
        let current_swapchain: Arc<Swapchain> = self.swapchain.as_ref().unwrap().0.clone();
        let new_swapchain: (Arc<Swapchain>, Vec<Arc<Image>>) = current_swapchain.recreate(
            SwapchainCreateInfo {
                image_extent: self.window_extend(),
                ..current_swapchain.create_info()
            }
        ).unwrap();
        self.swapchain = Some(new_swapchain);
        //rebuild dependencies
        self.build_framebuffers();
        self.build_pipeline();
        self.build_command_buffers();
        self.swapchain_invalid = false;
    }

    fn submit_drawcall(&mut self) {
        let swapchain: Arc<Swapchain> = self.swapchain.as_ref().unwrap().0.clone();
        let (device, queue) = self.vk_virtual_gpu.as_ref().unwrap().clone();
        let command = self.command_buffers.as_ref().unwrap().clone();

        //first acquire the image to draw on from the swapchain
        let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None)
                    .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        self.swapchain_invalid = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}"),
                };
        if suboptimal { self.swapchain_invalid = true; }

        println!("rendering");
        //FUTURE SHENNANIGANS
        let execution = sync::now(device.clone())
                .join(acquire_future)
                .then_execute(queue[0].clone(), command[image_i as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    queue[0].clone(), 
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i))
                .then_signal_fence_and_flush();

        execution.unwrap().wait(None).unwrap();

        //uncomment when I figure out how to do frames in flight
        //self.event_loop_proxy.as_ref().unwrap().send_event(UserEvent::Render).unwrap();
    }

    fn window_extend(&self) -> [u32; 2] {
        let window = self.window_main.as_ref().unwrap().0.clone();
        return [window.inner_size().width, window.inner_size().height]
    }

    fn render_format_or(&mut self, fallback: Format) -> Format {
        //since the render output image might not exist or be valid
        if self.swapchain.is_some() {
            self.swapchain.as_ref().unwrap().0.image_format()
        } else { fallback }
    }
}

impl ApplicationHandler<UserEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_main.is_none() {
            println!("pre window build");
            self.build_window(event_loop);
            println!("post window build");

            if self.vk_gpu.is_none() { self.initialize_gpu(); }
            if self.vk_virtual_gpu.is_none() { self.initialize_virtual_gpu(); }
            

            self.memory_allocator = Some(Arc::from(StandardMemoryAllocator::new_default(self.vk_virtual_gpu.as_ref().unwrap().0.clone())));
            self.command_allocator = Some(StandardCommandBufferAllocator::new(
                self.vk_virtual_gpu.as_ref().unwrap().0.clone(),
                 StandardCommandBufferAllocatorCreateInfo::default()));

            self.vert_buffer = Some(default_vertex_buffer(self.memory_allocator.as_ref().unwrap().clone()));

            self.build_swapchain();
            self.build_renderpass();
            self.build_framebuffers();
            self.compile_shaders();
            self.build_pipeline();
            self.build_command_buffers();
            
            self.event_loop_proxy.as_ref().unwrap().send_event(UserEvent::Render).unwrap();
        }
        //this is the start of the app
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        println!("user event received");
        match event {
            UserEvent::Render => { 
                if self.swapchain_invalid { self.rebuild_swapchain(); } 
                self.submit_drawcall(); 
            }
        }
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        match event {
            WindowEvent::CloseRequested => { event_loop.exit() }
            WindowEvent::RedrawRequested => { println!("Redraw Request"); self.swapchain_invalid = true; }
            _ => { }
        }
        
    }
}


//Sets up connection to GPU
fn initialize_vulkano(event_loop : &EventLoop<UserEvent>) -> Arc<Instance> {
    let vk_library = VulkanLibrary::new().expect("failed to make vulkan library"); 
    
    //get gpu driver (instance)
    let vk_instance_parameters: InstanceCreateInfo = InstanceCreateInfo {
        enabled_extensions: Surface::required_extensions(&event_loop), //swapchain extensions
        ..Default::default()
    };
    let vk_driver: Arc<Instance> = Instance::new(
        vk_library.clone(),
        vk_instance_parameters)
        .expect("Failed to create vulkan instance");

    //we can't locate a device until we've made our window, so this is all we can do for now
    return vk_driver;
}

//find an optimal GPU to use and a graphics queue index to use
fn locate_device(vk_driver : Arc<Instance>, extensions : &DeviceExtensions, window_surface : Arc<Surface>) 
-> Option<(Arc<PhysicalDevice>, Vec<u32>)> {
    use vulkano::device::physical::PhysicalDeviceType::*;
    
    //get all devices
    //select one to be the logical device
    let graphics_processors : Vec<Arc<PhysicalDevice>> = vk_driver.enumerate_physical_devices()
        .unwrap()
        .collect();

    let selected_gpu: Option<(Arc<PhysicalDevice>, Vec<u32>)> = graphics_processors
        .into_iter()
        .filter(|gpu | {
            gpu.supported_extensions().contains(&extensions)
        })
        .map(|gpu| {
            let queue_indices : Vec<u32> = gpu.queue_family_properties().iter().enumerate()
                .filter(|(index, family)| {
                    family.queue_flags.contains(QueueFlags::GRAPHICS) 
                        && gpu.surface_support(*index as u32, &window_surface).unwrap_or(false)
                })
                .map(|(index, family)| {
                index as u32
            }).collect();

            (gpu, queue_indices)
        })
        .filter(|(gpu, queue_indices)| {
            queue_indices.len() > 0
        })
        .min_by_key(|(gpu, queue_indices)| {
            match gpu.properties().device_type{
                DiscreteGpu => 0,
                IntegratedGpu => 1,
                VirtualGpu => 2,
                Cpu => 3,
                _ => 4
            }
        });

    return selected_gpu;
}

fn default_vertex_buffer(mem_allocator : Arc<dyn MemoryAllocator>) -> Subbuffer<[Vert]>{
    let vert1 = Vert { position: [-0.5, -0.5] };
    let vert2 = Vert { position: [0.0, 0.5] };
    let vert3 = Vert { position: [0.5, -0.25] };

    return Buffer::from_iter(
        mem_allocator, 
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        }, 
        AllocationCreateInfo{
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        }, 
        vec![vert1, vert2, vert3])
        .unwrap();
}
