use std::error::Error;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use bitflags::bitflags;
use gl::types::*;
use glutin::{
    context::PossiblyCurrentContext,
    display::{GetGlDisplay, GlDisplay},
};

mod atlas;
mod glyph_cache;
mod shader;

use glyph_cache::GlyphCache;
use shader::{ShaderError, ShaderProgram};

bitflags! {
    #[repr(C)]
    struct RenderingGlyphFlags: u8 {
        const COLORED   = 0b0000_0001;
        const WIDE_CHAR = 0b0000_0010;
    }
}

enum RenderingPass {
    Background = 0,
    Foreground = 1,
}

macro_rules! cstr {
    ($s:literal) => {
        // This can be optimized into an no-op with pre-allocated NUL-terminated bytes.
        unsafe { std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr().cast()) }
    };
}

static TEXT_SHADER_F: &str = include_str!("../../res/text.f.glsl");
static TEXT_SHADER_V: &str = include_str!("../../res/text.v.glsl");

const BATCH_MAX: usize = 0x1_0000;
static GL_FUNS_LOADED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
#[repr(C)]
struct InstanceData {
    // coords
    col: u16,
    row: u16,

    // glyph offset
    left: i16,
    top: i16,

    // glyph size
    width: i16,
    height: i16,

    // uv info
    uv_left: f32,
    uv_bot: f32,
    uv_width: f32,
    uv_height: f32,

    // foreground
    r: u8,
    g: u8,
    b: u8,

    // cell flags
    cell_flags: RenderingGlyphFlags,

    //background
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    bg_a: u8,
}

struct TextRenderBatch {
    pub tex: GLuint,
    instances: Vec<InstanceData>,
}

impl TextRenderBatch {
    #[inline]
    pub fn new() -> Self {
        Self {
            tex: 0,
            instances: Vec::with_capacity(BATCH_MAX),
        }
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.instances.len() == BATCH_MAX
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.instances.len() == 0
    }

    pub fn clear(&mut self) {
        self.tex = 0;
        self.instances.clear();
    }

    pub fn render(&mut self) {}

    // pub fn add_item(&mut self, glyph: &Glyph) {
    //     if !self.is_empty() && self.tex != glyph.tex_id {
    //         self.render();
    //         self.tex = glyph.tex_id;
    //     }

    //     let mut cell_flags = RenderingGlyphFlags::empty();

    //     let instance = InstanceData {
    //         col: 0,
    //         row: 0,
    //         left: glyph.left,
    //         top: glyph.top,
    //         width: glyph.width,
    //         height: glyph.height,
    //         uv_left: glyph.uv_left,
    //         uv_bot: glyph.uv_bot,
    //         uv_width: glyph.uv_width,
    //         uv_height: glyph.uv_height,
    //         r: 255,
    //         g: 255,
    //         b: 255,
    //         cell_flags,
    //         bg_r: 0,
    //         bg_g: 0,
    // bg_b: 0,
    // bg_a: 0,
    // };

    // self.instances.push(instance);

    // if self.full() {
    // self.render();
    // }
    // }
}

pub struct Renderer {
    program: TextShaderProgram,
    vao: GLuint,
    ebo: GLuint,
    vbo_instance: GLuint,

    glyph_cache: GlyphCache,
    text_batch: TextRenderBatch,
}

impl Renderer {
    pub fn new(
        context: &PossiblyCurrentContext,
    ) -> Result<Self, Box<dyn Error>> {
        if !GL_FUNS_LOADED.swap(true, Ordering::Relaxed) {
            let gl_display = context.display();
            gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });
        };

        // unsafe {
        // let version = CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8).to_str()?;
        // println!("{}", version);
        // }

        let program = TextShaderProgram::new()?;
        let mut vao: GLuint = 0;
        let mut ebo: GLuint = 0;
        let mut vbo_instance: GLuint = 0;

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);

            gl::DepthMask(gl::FALSE);

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut ebo);
            gl::GenBuffers(1, &mut vbo_instance);
            gl::BindVertexArray(vao);

            let indices: [u8; 6] = [0, 1, 3, 1, 2, 3];

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (6 * size_of::<u8>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_instance);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (BATCH_MAX * size_of::<InstanceData>()) as isize,
                ptr::null(),
                gl::STREAM_DRAW,
            );

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let mut index = 0;
            let mut size = 0;

            macro_rules! add_attr {
                ($count:expr, $gl_type:expr, $type:ty) => {
                    gl::VertexAttribPointer(
                        index,
                        $count,
                        $gl_type,
                        gl::FALSE,
                        size_of::<InstanceData>() as i32,
                        size as *const _,
                    );
                    gl::EnableVertexAttribArray(index);
                    gl::VertexAttribDivisor(index, 1);

                    #[allow(unused_assignments)]
                    {
                        size += $count * size_of::<$type>();
                        index += 1;
                    }
                };
            }

            // coords
            add_attr!(2, gl::UNSIGNED_SHORT, u16);

            // glyph offset and size
            add_attr!(4, gl::SHORT, i16);

            // uv info
            add_attr!(4, gl::FLOAT, f32);

            // color
            add_attr!(3, gl::UNSIGNED_BYTE, u8);

            // background color.
            add_attr!(4, gl::UNSIGNED_BYTE, u8);

            // clean up
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        let text_batch = TextRenderBatch::new();

        let mut glyph_cache = GlyphCache::new(12.0)?;

        // cache all basic ascii characters
        // for i in 32u8..=126u8 {
        //     glyph_cache.get(i as char);
        // }

        Ok(Self {
            program,
            vao,
            ebo,
            vbo_instance,
            glyph_cache,
            text_batch,
        })
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let scale_x = 2. / width;
        let scale_y = -2. / height;
        let offset_x = -1.;
        let offset_y = 1.;

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            gl::UseProgram(self.program.id());

            gl::Uniform4f(
                self.program.u_projection,
                offset_x,
                offset_y,
                scale_x,
                scale_y,
            );
            gl::UseProgram(0);
        }
    }

    pub fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo_instance);
            gl::DeleteBuffers(1, &self.ebo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

#[derive(Debug)]
pub struct TextShaderProgram {
    program: ShaderProgram,
    u_projection: GLint,
    u_cell_dim: GLint,
    u_rendering_pass: GLint,
}

impl TextShaderProgram {
    pub fn new() -> Result<Self, ShaderError> {
        let program = ShaderProgram::new(TEXT_SHADER_V, TEXT_SHADER_F)?;
        Ok(Self {
            u_projection: program.get_uniform_location(cstr!("projection"))?,
            u_cell_dim: program.get_uniform_location(cstr!("cellDim"))?,
            u_rendering_pass: program
                .get_uniform_location(cstr!("renderingPass"))?,
            program,
        })
    }

    pub fn id(&self) -> GLuint {
        self.program.id()
    }

    unsafe fn set_cell_dim(&self, cell_width: f32, cell_height: f32) {
        gl::Uniform2f(self.u_cell_dim, cell_width, cell_height);
    }

    unsafe fn set_rendering_pass(&self, rendering_pass: RenderingPass) {
        gl::Uniform1i(self.u_rendering_pass, rendering_pass as i32);
    }
}
