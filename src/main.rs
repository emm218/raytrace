use std::error::Error;

use winit::{
    event::{ Event, WindowEvent },
    event_loop::EventLoop,
    window::Window,
};

mod display;

use display::Display;

fn main() -> Result<(),Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;
    
    let display = Display::new(window)?;
    
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                control_flow.set_exit();
            }
            _ => ()
        }
    })
}
