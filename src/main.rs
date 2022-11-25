use glfw::{Action, Context, Key};



fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
        .expect("failed to initialize glfw");

    let (mut window, events) = 
        glfw.create_window(300, 300, "raytrace", glfw::WindowMode::Windowed)
        .expect("failed to create glfw window");

    window.set_key_polling(true);
    window.make_current();
    
    gl::load_with(|s| window.get_proc_address(s));
    
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        window.swap_buffers();
        
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Q, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        glfw::WindowEvent::Size(width, height) => {
            unsafe {
                gl::Viewport(0, 0, width, height);
            }
        }
        _ => {} 
    }
}
