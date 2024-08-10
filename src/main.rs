use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, 
    command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, 
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, 
    device::{
        physical::PhysicalDevice, 
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, format::Format, image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType}, Image, ImageSubresourceRange, ImageUsage}, instance::{Instance, InstanceCreateInfo}, library::VulkanLibrary, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, shader::{self, ShaderModule}, swapchain::{ColorSpace, CompositeAlpha, PresentMode, Surface, Swapchain, SwapchainCreateInfo}, NonExhaustive 
};
use winit::{
    application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowAttributes}};
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
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
    }
}

static WINDOW_INIT_WIDTH : u32 = 600;
static WINDOW_INIT_HEIGHT : u32 = 400;

fn main() {
    //1 Connect to GPU
    
    let event_loop: EventLoop<_> = EventLoop::new().unwrap();

    let vulkan_driver = initialize_vulkano(&event_loop);

    let mut render_app: Application = Application {
        vk_driver: vulkan_driver,
        window_create_info: Window::default_attributes()
            .with_blur(false)
            .with_inner_size(LogicalSize::new(WINDOW_INIT_WIDTH, WINDOW_INIT_HEIGHT))
            .with_resizable(false)
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

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vert {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2]
}

struct Application {
    vk_driver : Arc<Instance>,
    vk_GPU : Option<(Arc<PhysicalDevice>, Vec<u32>)>,
    vk_virtual_GPU : Option<(Arc<Device>, Vec<Arc<Queue>>)>,

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

    render_pipeline : Option<Arc<GraphicsPipeline>>
}

impl Default for Application {
    fn default() -> Self {
        Self { 
            vk_driver: {vulkano::instance::Instance::new(vulkano::VulkanLibrary::new().unwrap(), InstanceCreateInfo::default()).unwrap()}, 
            vk_GPU: None,
            vk_virtual_GPU: None, 
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
            render_pipeline: None }
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
        self.vk_GPU = Some(device);
    }

    fn initialize_virtual_gpu(&mut self) {
        let (gpu, queue_indices) = self.vk_GPU.as_ref().unwrap().clone();
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

        self.vk_virtual_GPU = Some((device, queues))
    }

    fn build_swapchain(&mut self) {
        let (gpu, queues) = self.vk_virtual_GPU.as_ref().unwrap();
        let (window, surface) = self.window_main.as_ref().unwrap();
        ;
        let capabilities = self.vk_virtual_GPU.as_ref().unwrap().0
            .physical_device()
            .surface_capabilities(surface, Default::default())
            .unwrap();
        let window_size : [u32; 2] = [window.inner_size().width, window.inner_size().height];
        let composite_alpha = capabilities.supported_composite_alpha.into_iter().next().unwrap();
        let format = self.vk_GPU.as_ref().unwrap().0
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
            self.vk_virtual_GPU.as_ref().unwrap().0.clone(),
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
        let gpu = self.vk_virtual_GPU.as_ref().unwrap().0.clone();
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

        let vk_device = self.vk_virtual_GPU.as_ref().unwrap().0.clone();

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

    fn render_format_or(&mut self, fallback: Format) -> Format {
        //since the render output image might not exist or be valid
        if self.swapchain.is_some() {
            self.swapchain.as_ref().unwrap().0.image_format()
        } else { fallback }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_main.is_none() {
            self.build_window(event_loop);

            if self.vk_GPU.is_none() { self.initialize_gpu(); }
            if self.vk_virtual_GPU.is_none() { self.initialize_virtual_gpu(); }
            
            self.build_swapchain();

            self.memory_allocator = Some(Arc::from(StandardMemoryAllocator::new_default(self.vk_virtual_GPU.as_ref().unwrap().0.clone())));
            self.command_allocator = Some(StandardCommandBufferAllocator::new(
                self.vk_virtual_GPU.as_ref().unwrap().0.clone(),
                 StandardCommandBufferAllocatorCreateInfo::default()));

            self.vert_buffer = Some(default_vertex_buffer(self.memory_allocator.as_ref().unwrap().clone()));

            self.build_renderpass();
            self.build_framebuffers();
            self.compile_shaders();
            self.build_pipeline();

            let mut command_builder = AutoCommandBufferBuilder::primary(
                self.command_allocator.as_ref().unwrap(), 
                self.vk_GPU.as_ref().unwrap().1[0],
                CommandBufferUsage::OneTimeSubmit).unwrap();

            command_builder
                .begin_render_pass(
                    RenderPassBeginInfo{
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(self.render_buffers.as_ref().unwrap()[0].clone())
                    }, 
                    SubpassBeginInfo{
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    }).unwrap()
                    .bind_pipeline_graphics(self.render_pipeline.as_ref().unwrap().clone())
                    .unwrap()
                    .bind_vertex_buffers(0, self.vert_buffer.as_ref().unwrap().clone())
                    .unwrap()
                    .draw(
                        3, 1, 0, 0, // 3 is the number of vertices, 1 is the number of instances
                    )
                    .unwrap()
                    .end_render_pass(SubpassEndInfo::default())
                    .unwrap();

            let commandbuffer = command_builder.build().unwrap();
        }
        //this is the start of the app
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        match event {
            WindowEvent::CloseRequested => { event_loop.exit() }
            WindowEvent::RedrawRequested => {}
            _ => { }
        }
        
    }
}


//Sets up connection to GPU
fn initialize_vulkano(event_loop : &EventLoop<()>) -> Arc<Instance> {
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
    let mut graphics_processors : Vec<Arc<PhysicalDevice>> = vk_driver.enumerate_physical_devices()
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
    let vert1 = Vert { position: [-1.0, -1.0] };
    let vert2 = Vert { position: [1.0, -1.0] };
    let vert3 = Vert { position: [-1.0, 1.0] };

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
