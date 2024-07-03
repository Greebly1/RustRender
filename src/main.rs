use std::{
    io::stdin,
    sync::Arc
};
use vulkano::device::{
    Device,
    physical::PhysicalDevice,
    QueueFlags,
    QueueCreateInfo,
    DeviceCreateInfo
};


fn main() {
    let mut terminal_input: String = String::new(); 

    let vulkan_lib = vulkano::library::VulkanLibrary::new()
        .expect("Failed to make default vulkan library"); //default vulkan library
    let vulkan_create_info = vulkano::instance::InstanceCreateInfo::default();
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

    let (render_device, mut render_queues) = Device::new(
        graphics_processor, 
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default() //im ganna be honest I dont fully understand this section of code
            }],
            ..Default::default()
        },
    ).expect("failed to create message queue with render device");

    let event_loop = winit::event_loop::EventLoop::new().unwrap();

    let mut app : Application = Application{
        window_main : None,
        window_create_info : winit::window::WindowAttributes::default()
    };

    println!("Initialization complete, press ENTER to begin");
    stdin().read_line(&mut terminal_input).unwrap();

    event_loop.run_app(&mut app).unwrap();
}

struct Application {
    //Mutable singleton that stores global data for our ApplicationHandler hooks to use
    window_main: Option<winit::window::Window>,
    window_create_info : winit::window::WindowAttributes
}

impl Application {

    fn main_window_id(&self) -> Option<winit::window::WindowId> {
        if self.window_main.is_some() {
            return Some(self.window_main.as_ref().unwrap().id());
        } else {
            return None;
        }
    }
}

impl winit::application::ApplicationHandler for Application {
    //This trait provides hooks into winit, so we can define custom behavior, think MonoBehavior in Unity
    //https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html 
    //some of these hooks only emit on certain platforms, like android or Mac

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window_main = Some(event_loop
            .create_window(self.window_create_info.clone())
            .expect("Failed to make window from attributes")
        );
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
            Moved(position) => { println!("Moving windows won't be included on wayland"); }

            CloseRequested => { 
                if self.window_main.is_some() {
                    let main_win_id = self.main_window_id().unwrap();
                    if main_win_id == window_id { event_loop.exit(); }
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
