use glfw::{
    Window, 
    Context
};
use vulkano::{
    VulkanLibrary,
    instance::{
        Instance,
        InstanceCreateInfo
    }
};

use std::{
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
    let mut frame_counter: u64 = 0;
    let sleep_time : Duration = Duration::new(0, 50000000);

    //GLFW for window creation
    let mut glfw_instance = glfw::init(handle_errors).unwrap();

    //VULKANO for vulkan drawing api
    let vk_library : Arc<VulkanLibrary> = VulkanLibrary::new()
        .expect("Failed to retrieve a valid vulkan library");
    let vk_instance = Instance::new(vk_library, InstanceCreateInfo::default())
        .expect("failed to create vulkan api instance");


    //glfw_instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi)); <-- I am pretty sure we need this so it doesnt make a opengl context. But with no api provided it throws errors
    //should also include window resizable hint

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