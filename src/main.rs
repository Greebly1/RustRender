use std::{
    io::stdin, sync::Arc
};
use vulkano::{
    device::{ 
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags
}, 
    image::ImageUsage, 
    instance::Instance, 
    swapchain::Swapchain,
    command_buffer::allocator::{
        StandardCommandBufferAllocatorCreateInfo,
        StandardCommandBufferAllocator
    },
    memory::allocator::{
        StandardMemoryAllocator,
        AllocationCreateInfo,
        MemoryTypeFilter
    },
    buffer::{
        Buffer,
        Subbuffer,
        BufferUsage,
        BufferCreateInfo
    }
};


fn main() {
    let mut terminal_input: String = String::new(); 

    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let vulkan_lib = vulkano::library::VulkanLibrary::new()
        .expect("Failed to make default vulkan library"); //default vulkan library
    let winit_extensions = vulkano::swapchain::Surface::required_extensions(&event_loop);
    let vulkan_create_info: vulkano::instance::InstanceCreateInfo = vulkano::instance::InstanceCreateInfo{
        application_name: Some(String::from("Rust_Render")),
        engine_name: Some(String::from("V8Engine")),
        enabled_extensions: winit_extensions,
        ..Default::default()
    };
    let vulkan = vulkano::instance::Instance::new(vulkan_lib, vulkan_create_info)
        .unwrap();

    let mut vk_graphics_processors : Vec<Arc<PhysicalDevice>> = vulkan.enumerate_physical_devices()
        .unwrap()
        .collect();
    assert!(vk_graphics_processors.len() > 0, "This machine does not have a vulkan compatible GPU");

    println!("Select a GPU to proceed");                
    for (index, vk_device) in vk_graphics_processors.iter().enumerate() { //prints the name of each GPU
        println!("{}. {}", index+1, vk_device.properties().device_name);
    } 

    stdin().read_line(&mut terminal_input).unwrap();
    terminal_input = terminal_input.trim().to_string();
    let user_selection = terminal_input.parse::<u8>().expect("You did not input a valid GPU ID");
    assert!(user_selection > 0 && user_selection <= (vk_graphics_processors.len() as u8), "You did not input a valid GPU ID");
    let graphics_processor : Arc<PhysicalDevice> = vk_graphics_processors.swap_remove((user_selection - 1) as usize);

    println!("Proceeding with selected graphics processor: {}", graphics_processor.properties().device_name);
    println!("Device supporting Version {:?}", graphics_processor.api_version());

    println!("Listing GPU queue families");
    for queue_family in graphics_processor.queue_family_properties() {
        println!("Found a queue family with {} queue channel(s)", queue_family.queue_count);
    }

    let queue_family_index : u32 = graphics_processor
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_queue_family_index , queue_family_properties)| {
            queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
        }).expect("This GPU has no open graphics queues") as u32;

    let device_extensions = DeviceExtensions{
        khr_swapchain: true,
        ..Default::default()
    };

    let (render_device, mut render_queues) = Device::new(
        graphics_processor, 
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default() 
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    ).expect("failed to create message queue with render device");

    let command_allocator_create_info : StandardCommandBufferAllocatorCreateInfo = StandardCommandBufferAllocatorCreateInfo::default();
    let vulkan_command_allocator : StandardCommandBufferAllocator = StandardCommandBufferAllocator::new(render_device.clone(), command_allocator_create_info);


    let mut app : Application = Application{
        window_main : None,
        window_create_info : winit::window::WindowAttributes::default(),

        vulkan_instance : vulkan,
        memory_allocator : Arc::from(StandardMemoryAllocator::new_default(render_device.clone())),
        graphics_processor : render_device,
        render_queues : render_queues.collect(),
        command_allocator : vulkan_command_allocator,

        buffer_src : None,
        buffer_dest : None
    };

    println!("Initialization complete, press ENTER to begin");
    stdin().read_line(&mut terminal_input).unwrap();

    event_loop.run_app(&mut app).unwrap();
}

struct MyWindowData {
    window: Arc<winit::window::Window>,
    render_surface: Arc<vulkano::swapchain::Surface>,

    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<vulkano::image::Image>>
}

struct Application {
    //Mutable singleton that stores global data for our ApplicationHandler hooks to use
    window_main: Option<MyWindowData>,
    window_create_info : winit::window::WindowAttributes,
    
    //when we make our winit application we will move all of the vulkan stuff into it because
    vulkan_instance : Arc<Instance>,
    graphics_processor : Arc<Device>,
    render_queues : Vec<Arc<vulkano::device::Queue>>,
    command_allocator : StandardCommandBufferAllocator,
    memory_allocator : Arc<dyn vulkano::memory::allocator::MemoryAllocator>,

    buffer_src : Option<Subbuffer<[i32]>>,
    buffer_dest : Option<Subbuffer<[i32]>>
}

impl Application {

    fn main_window_id(&self) -> Option<winit::window::WindowId> {
        if self.window_main.is_some() {
            return Some(self.window_main.as_ref().unwrap().window.id());
        } else {
            return None;
        }
    }
}

impl MyWindowData {

}

impl winit::application::ApplicationHandler for Application {
    //This trait provides hooks into winit, so we can define custom behavior, think MonoBehavior in Unity
    //https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html 
    //some of these hooks only emit on certain platforms, like android or Mac

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        //make the window, the render surface, and the swapchain
        let main_window: Arc<winit::window::Window> = Arc::new(event_loop
            .create_window(self.window_create_info.clone())
            .expect("Failed to make window from attributes"));

        let main_surface: Arc<Surface> = 
            Surface::from_window(self.vulkan_instance.clone(), main_window.clone())
            .unwrap();
        
        use vulkano::swapchain::*;
        //platform and hardware specific capabilities
        let surface_capabiities = self.graphics_processor
            .physical_device()
            .surface_capabilities(&main_surface, Default::default())
            .unwrap();
        let image_resolution = surface_capabiities.current_extent.unwrap_or([640,480]);
        let transform = surface_capabiities.current_transform;
        let (format, color_space) = self.graphics_processor
            .physical_device()
            .surface_formats(&main_surface, Default::default())
            .unwrap()[0]; //use first image format
            //TODO: ideally pick an image format

        let swapchain_create_info = SwapchainCreateInfo {
            flags: SwapchainCreateFlags::empty(),
            min_image_count: 2, //double buffer
            image_format: format,
            image_color_space: color_space,
            image_extent: image_resolution,
            pre_transform: transform,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode: PresentMode::Mailbox, //v-sync
            ..Default::default()
        };
        let (main_swapchain, swapchain_imgs) = Swapchain::new(
                self.graphics_processor.clone(), 
                main_surface.clone(), 
                swapchain_create_info)
            .unwrap();

        let main_window_data = MyWindowData {
            window: main_window,
            render_surface: main_surface,
            swapchain: main_swapchain,
            swapchain_images: swapchain_imgs
        };

        self.window_main = Some(main_window_data);

        let source_content_hardCoded : Vec<i32> = (0..64).collect();
        self.buffer_src = Some(Buffer::from_iter(
            self.memory_allocator.clone(), 
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            }, 
            AllocationCreateInfo {
                memory_type_filter : MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            }, 
            source_content_hardCoded).unwrap());

        let destination_content_hardCoded : Vec<i32> = (0..64).map(|_| 0).collect();
        self.buffer_dest = Some(
            Buffer::from_iter(
                self.memory_allocator.clone(), 
                BufferCreateInfo {
                    usage : BufferUsage::TRANSFER_DST,
                    ..Default::default()
                }, 
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            }, 
                destination_content_hardCoded
            ).unwrap()
        )
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
        use winit::event::WindowEvent::*;

        match event {
            Resized(size) => { }
            ScaleFactorChanged { scale_factor, inner_size_writer } => { }
            RedrawRequested => { }

            #[cfg(not(any(ios_platform, android_platform, web_platform, wayland_platform)))]
            Moved(position) => {  }

            CloseRequested => { 
                if self.main_window_id().unwrap() == window_id {
                    event_loop.exit();
                }
                }
            Destroyed => { }
            Focused(is_focused) => { }
            KeyboardInput { device_id, event, is_synthetic } => { }
            ModifiersChanged(key_modifier) => { }
            CursorMoved { device_id, position } => { }
            CursorEntered { device_id } => { }
            CursorLeft { device_id } => { }
            MouseWheel { device_id, delta, phase } => { }
            MouseInput { device_id, state, button } => { }
            _ => { } //unsupported event
        }
    }

    fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
        
    }

    //eventloop_proxy is used for custom events we can send programmatically from other threads
    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
        //this is kind of how other threads can interface with our application   
    }
    
    fn device_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    }

    #[cfg(any(android_platform, ios_platform, web_platform))]
    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        
    }

    #[cfg(any(android_platform, ios_platform))]
    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        
    }
}

