use std::sync::atomic::{ AtomicBool, Ordering };
use std::ffi::{ CString };

use winit::dpi::PhysicalSize;
use glutin::{
    context::PossiblyCurrentContext,
    display::{ GlDisplay, GetGlDisplay },
};

static GL_FUNS_LOADED: AtomicBool = AtomicBool::new(false);

pub fn init(context: &PossiblyCurrentContext) {
    if !GL_FUNS_LOADED.swap(true, Ordering::Relaxed) {
        let gl_display = context.display();
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });
    };

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }
}

pub fn resize(size: PhysicalSize<u32>) {
    unsafe {
        gl::Viewport(0, 0, size.width as i32, size.height as i32);
    }
}

pub fn clear() {
    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
}

pub fn finish() {
    unsafe { gl::Finish(); }
}


