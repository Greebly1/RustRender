use glfw::{Window, Context};
use std::thread::sleep;
use std::time::Duration;


fn main() {
    let mut glfw_instance = glfw::init(handle_errors).unwrap();

    let mut frame_counter: u64 = 0;

    let (win_width, win_height) : (u32, u32) = (500, 300);
    let win_title : &str = "Rust3D";
    let sleep_time : Duration = Duration::new(0, 50000000);

    //glfw_instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi)); <-- I am pretty sure we need this so it doesnt make a opengl context. But with no api provided it throws errors
    //should also include window resizable hint

    let (mut window, event_loop) = glfw_instance
    .create_window(win_width, win_height, win_title, glfw::WindowMode::Windowed).unwrap();

    window.set_key_polling(true);
    window.make_current();

    while !window.should_close() {

        glfw_instance.poll_events();

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
        _ => { }
    }
}

fn handle_errors (err: glfw::Error, message : String) {
    panic!("GLFW failed: {:?}, {:?}", err, message);
}