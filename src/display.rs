use std::error::Error;
use std::num::NonZeroU32;

use winit::{
    window::Window,
    platform::unix,
    dpi::PhysicalSize,
};
use raw_window_handle::{
    RawDisplayHandle,
    HasRawDisplayHandle,
    RawWindowHandle,
    HasRawWindowHandle,
};
use glutin::{
    config:: { Config, ConfigTemplateBuilder, GetGlConfig },
    context::{ 
        ContextApi,
        ContextAttributesBuilder,
        NotCurrentContext,
        NotCurrentGlContextSurfaceAccessor,
        PossiblyCurrentContext,
        Version 
    },
    display::{ 
        Display as GlutinDisplay, 
        DisplayApiPreference, 
        GlDisplay, 
        GetGlDisplay 
    },
    error::Result as GlutinResult,
    prelude::GlSurface,
    surface::{ Surface, SurfaceAttributesBuilder, WindowSurface },
};

use crate::renderer;

pub struct Display {
    pub window: Window,
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
}

impl Display {
    pub fn new(window: Window) -> Result<Self, Box<dyn Error>> {
        let display = create_gl_display(window.raw_display_handle())?;
        let config = pick_gl_config(&display, None)?;
        let context = create_gl_context(&display, &config, None)?;
        let surface = create_gl_surface(&context, window.inner_size(), window.raw_window_handle())?;
        
        let context = context.make_current(&surface)?;
        
        renderer::init(&context);
        
        renderer::resize(window.inner_size());

        renderer::clear();

        surface.swap_buffers(&context).expect("failed to swap buffers.");
        renderer::finish();

        Ok(Self {
            window,
            context,
            surface,
        })
    }
}

fn create_gl_display(
    display_handle: RawDisplayHandle
) -> GlutinResult<GlutinDisplay> {
    
    let preference = DisplayApiPreference::GlxThenEgl(
        Box::new(unix::register_xlib_error_hook));
    
    unsafe { GlutinDisplay::new(display_handle, preference) }
}

fn pick_gl_config(
    gl_display: &GlutinDisplay,
    window_handle: Option<RawWindowHandle>,
) -> Result<Config, String> {
    
    let mut default_config = ConfigTemplateBuilder::new();

    if let Some(window_handle) = window_handle {
        default_config = 
            default_config.compatible_with_native_window(window_handle);
    }
    
    let gl_config = unsafe {
        gl_display.find_configs(default_config.build())
            .ok()
            .and_then(|mut configs| configs.next())
    };

    if let Some(gl_config) = gl_config {
        return Ok(gl_config);
    }

    Err(String::from("failed to find a valid gl config"))
}

fn create_gl_context(
    gl_display: &GlutinDisplay,
    gl_config: &Config,
    window_handle: Option<RawWindowHandle>
) -> GlutinResult<NotCurrentContext> {
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .build(window_handle);
    
    unsafe { gl_display.create_context(&gl_config, &context_attributes) }
}

fn create_gl_surface(
    gl_context: &NotCurrentContext,
    size: PhysicalSize<u32>,
    window_handle: RawWindowHandle
) -> GlutinResult<Surface<WindowSurface>> {
    
    let gl_display = gl_context.display();
    let gl_config = gl_context.config();

    let surface_attributes =
        SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window_handle,
            NonZeroU32::new(size.width).expect("width must be non-zero"),
            NonZeroU32::new(size.height).expect("height must be non-zero")
        );

    unsafe { gl_display.create_window_surface(&gl_config, &surface_attributes) }
} 
