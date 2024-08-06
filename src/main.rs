use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::{CommandBufferAllocator, StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, device::{
        physical::PhysicalDevice, 
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags}, format::Format, image::{
        view::ImageView, 
        Image, ImageCreateInfo, ImageType, ImageUsage}, instance::{Instance, InstanceCreateInfo}, library::VulkanLibrary, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass}, shader::{self, ShaderModule}, swapchain::{ColorSpace, CompositeAlpha, PresentMode, Surface, Swapchain, SwapchainCreateInfo}, NonExhaustive 
};
use winit::{
    application::ApplicationHandler, 
    event::WindowEvent, 
    event_loop::{ActiveEventLoop, EventLoop}, 
    window::{Window, WindowAttributes}};
use std::sync::Arc;

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

fn main() {
    //1 Connect to GPU
    

    let event_loop: EventLoop<_> = EventLoop::new().unwrap();



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
    render_buffer : Option<Arc<Framebuffer>>,
    render_img : Option<(Arc<Image>, Arc<ImageView>)>,

    vert_shader : Arc<ShaderModule>,
    frag_shader : Arc<ShaderModule>,

    render_pipeline : Option<Arc<GraphicsPipeline>>
}

impl Application {
    fn CreateWindow(&mut self, event_loop : &ActiveEventLoop) {
        let new_window = Arc::new(event_loop.create_window(self.window_create_info.clone())
        .unwrap());

        //https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
        //Says that making a surface is platform specific, so we might need to do conditional compilation for certain platforms
        let window_surface = Surface::from_window(self.vk_driver.clone(), new_window.clone())
                .unwrap();

        self.window_main = Some((new_window, window_surface))
    }

    fn InitGPU(&mut self) {
        let device_extensions = DeviceExtensions{
            khr_swapchain: true,
            ..Default::default()
        };
        let surface = self.window_main.as_ref().unwrap().1.clone();

        let device = LocateDevice(self.vk_driver.clone(), &device_extensions, surface).unwrap();
    }

    fn InitVirtualGPU(&mut self) {
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

    fn BuildSwapchain(&mut self) {
        let (gpu, queues) = self.vk_virtual_GPU.as_ref().unwrap();
        let (window, surface) = self.window_main.as_ref().unwrap();
        ;
        let capabilities = self.vk_virtual_GPU.as_ref().unwrap().0
            .physical_device()
            .surface_capabilities(surface, Default::default())
            .unwrap();

        let swapchain_parameters = SwapchainCreateInfo {
            min_image_count: capabilities.min_image_count + 1,
            image_format: Format::R8G8B8A8_SRGB,
            image_color_space: ColorSpace::SrgbNonLinear,
            image_extent: capabilities.current_extent.unwrap_or([640, 480]),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode : PresentMode::Mailbox,
            ..Default::default()
        };

        let (swapchain, swapchain_images) = Swapchain::new(gpu.clone(), surface.clone(), swapchain_parameters).unwrap();
        self.swapchain = Some((swapchain, swapchain_images));
    }

    fn BuildRenderPass(&mut self) {
        let window_size = self.window_main.as_ref().unwrap().0.inner_size();

        let render_image: Arc<Image> = Image::new(
            self.memory_allocator.as_ref().unwrap().clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d, //2 dimensional
                format: Format::R8G8B8A8_UNORM,
                extent: [window_size.width, window_size.height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            }
        ).unwrap();

        self.render_img = Some((render_image.clone(), ImageView::new_default(render_image.clone()).unwrap()));

        let render_pass : Arc<RenderPass> = vulkano::single_pass_renderpass!(
            self.vk_virtual_GPU.as_ref().unwrap().0.clone(),
            attachments: {
                color: {
                    format: self.RenderFormatOr(Format::R8G8B8A8_UNORM),
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

        let frame_buf = Framebuffer::new(
            self.render_pass.as_ref().unwrap().clone(), 
            FramebufferCreateInfo {
                attachments: vec![self.render_img.as_ref().unwrap().1.clone()],
                ..Default::default()
            }
        ).unwrap();
        
    }

    fn BuildPipeline(&mut self) {
        let window_size = self.window_main.as_ref().unwrap().0.inner_size();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [f32::from_bits(window_size.width), f32::from_bits(window_size.height)],
            depth_range: 0.0..=1.0, //<-- since the format is UNORM I think
        };
        
        let vs = self.vert_shader.clone().entry_point("main").unwrap();
        let fs = self.frag_shader.clone().entry_point("main").unwrap();

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

    fn RenderFormatOr(&mut self, fallback: Format) -> Format {
        //since the render output image might not exist or be valid
        if self.render_img.is_some() {
            self.render_img.as_ref().unwrap().0.format()
        } else { fallback }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_main.is_none() {
            self.CreateWindow(event_loop);

            if self.vk_GPU.is_none() { self.InitGPU(); }
            if self.vk_virtual_GPU.is_none() { self.InitVirtualGPU(); }
            
            self.BuildSwapchain();

            self.memory_allocator = Some(Arc::from(StandardMemoryAllocator::new_default(self.vk_virtual_GPU.as_ref().unwrap().0.clone())));
            self.command_allocator = Some(StandardCommandBufferAllocator::new(
                self.vk_virtual_GPU.as_ref().unwrap().0.clone(),
                 StandardCommandBufferAllocatorCreateInfo::default()));

            self.vert_buffer = Some(DefaultVertexBuffer(self.memory_allocator.as_ref().unwrap().clone()));

            self.BuildRenderPass();
            self.BuildPipeline();

            let mut command_builder = AutoCommandBufferBuilder::primary(
                self.command_allocator.as_ref().unwrap(), 
                self.vk_GPU.as_ref().unwrap().1[0],
                CommandBufferUsage::OneTimeSubmit).unwrap();

            command_builder
                .begin_render_pass(
                    RenderPassBeginInfo{
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(self.render_buffer.as_ref().unwrap().clone())
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
            _ => { }
        }
        
    }
}


//Sets up connection to GPU
fn InitializeVulkano(event_loop : &EventLoop<()>) -> Arc<Instance> {
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
fn LocateDevice(vk_driver : Arc<Instance>, extensions : &DeviceExtensions, window_surface : Arc<Surface>) 
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

fn DefaultVertexBuffer(mem_allocator : Arc<dyn MemoryAllocator>) -> Subbuffer<[Vert]>{
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