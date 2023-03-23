use std::error::Error;

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

mod display;
mod renderer;

use display::Display;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;

    let mut display = Display::new(&window)?;
    let mut command_buffer = Vec::with_capacity(128);

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::ReceivedCharacter(c) => {
                    received_char(c, &mut command_buffer)
                }
                WindowEvent::Resized(size) => display.resize(size),
                _ => (),
            },
            Event::RedrawRequested(_) => {
                display.draw();
            }
            _ => (),
        }
    })
}

fn received_char(c: char, command_buffer: &mut Vec<char>) {
    match c {
        '\x08' => {
            command_buffer.pop();
        }
        '\n' => {
            println!("{:?}", command_buffer);
            command_buffer.clear()
        }
        _ => command_buffer.push(c),
    }
}
