use glfw::{
    Window, 
    Context
};
use vulkano::{
    VulkanLibrary,
    instance::{
        Instance,
        InstanceCreateInfo
    },
    device::{
        Device,
        DeviceCreateInfo,
        physical::PhysicalDevice,
        QueueCreateInfo,
        QueueFlags
    }
};
use std::{
    io::stdin, 
    sync::Arc, 
    thread::sleep, 
    time::Duration
};

//Placing window data here to keep code clean
//In the future we will need to set the win_width/height dynamically probably
static WIN_TITLE : &str = &"bruh";
static WIN_WIDTH : u32 = 500;
static WIN_HEIGHT : u32 = 300;

fn main() {
    let mut terminal_input: String = String::new();
    let mut frame_counter: u64 = 0;
    let sleep_time : Duration = Duration::new(0, 50000000);

    //GLFW for window creation
    let mut glfw_instance = glfw::init(handle_errors).unwrap();
    //glfw_instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi)); <-- I am pretty sure we need this so it doesnt make a opengl context. But with no api provided it throws errors
    //should also include window resizable hint

    //VULKANO for vulkan drawing api
    let vk_library : Arc<VulkanLibrary> = VulkanLibrary::new()
        .expect("Failed to retrieve a valid vulkan library/DLL");
    let vk_instance = Instance::new(vk_library, InstanceCreateInfo::default())
        .expect("failed to create a vulkan api instance");
    let mut vk_graphics_processors : Vec<Arc<PhysicalDevice>> = vk_instance.enumerate_physical_devices()
        .expect("Failed to retrieve a vulkan compatible graphics processors")
        .collect(); //.enumerate_physical_devices returns an iterator, but I just want a collection. .collect() turns an iterator into a collection of what it was iterating over
    
    assert!(vk_graphics_processors.len() > 0, "Your machine has no vulkan compatible graphics processors");

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

    println!("Press enter to start window and event loop");
    stdin().read_line(&mut terminal_input).unwrap();

    let (mut window, event_loop) = glfw_instance
        .create_window(WIN_WIDTH, WIN_HEIGHT, WIN_TITLE, glfw::WindowMode::Windowed)
        .unwrap();
    window.set_key_polling(true);
    window.make_current();

    while !window.should_close() { //Game loop

        glfw_instance.poll_events(); //this function blocks execution sometimes during input

        for (_, event) in glfw::flush_messages(&event_loop) {
            handle_events(&mut window, event);
        }

        sleep(sleep_time);

        //DEBUG: debug counter
        frame_counter = frame_counter + 1;
        println!("Tick {}", frame_counter);
    }
}

fn handle_events (window: &mut Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Close => { window.set_should_close(true) }
        _ => { } //default => do nothing
    }
}

fn handle_errors (err: glfw::Error, message : String) {
    panic!("GLFW failed: {:?}, {:?}", err, message);
}