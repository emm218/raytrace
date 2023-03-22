use std::ffi::CStr;
use std::fmt;


use gl::types::*;

#[derive(Debug)]
pub struct ShaderProgram(GLuint);

impl ShaderProgram {
    pub fn new( 
        vertex_shader: &'static str,
        fragment_shader: &'static str,
    ) -> Result<Self, ShaderError> {
        let vertex_shader = Shader::new(
            gl::VERTEX_SHADER, 
            vertex_shader)?;
        let fragment_shader = Shader::new(
            gl::FRAGMENT_SHADER, 
            fragment_shader)?;

        let program = unsafe { Self(gl::CreateProgram()) };
        let mut success: GLint = 0;
        unsafe {
            gl::AttachShader(program.id(), vertex_shader.id());
            gl::AttachShader(program.id(), fragment_shader.id());
            gl::LinkProgram(program.id());
            gl::GetProgramiv(program.id(), gl::LINK_STATUS, &mut success);
        }

        if success != GLint::from(gl::TRUE) {
            Err(ShaderError::Link(get_program_info_log(program.id())))
        } else {
            Ok(program)
        }
    }

    pub fn get_uniform_location(&self, name: &'static CStr
        ) -> Result<GLint, ShaderError> {
        let ret = unsafe { gl::GetUniformLocation(self.id(), name.as_ptr()) };
        if ret < 0 {
            return Err(ShaderError::Uniform(name));
        }
        Ok(ret)
    }

    pub fn id(&self) -> GLuint {
        self.0
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.0) }
    }
}

#[derive(Debug)]
pub struct Shader(GLuint);

impl Shader {
    fn new(
        kind: GLenum,
        source: &'static str,
    ) -> Result<Self, ShaderError> {
        let length = source.len() as GLint;

        let shader = unsafe { Self(gl::CreateShader(kind)) };

        let mut success: GLint = 0;
        unsafe {
            gl::ShaderSource(
                shader.id(),
                1,
                &source.as_ptr().cast(),
                &length,
            );
            gl::CompileShader(shader.id());
            gl::GetShaderiv(shader.id(), gl::COMPILE_STATUS, &mut success);
        }

        if success != GLint::from(gl::TRUE) {
            Err(ShaderError::Compile(get_shader_info_log(shader.id())))
        } else {
            Ok(shader)
        }
    }
    
    pub fn id(&self) -> GLuint {
        self.0
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) }
    }
}

fn get_program_info_log(program: GLuint) -> String {
    let mut max_length: GLint = 0;
    unsafe {
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut max_length);
    }

    let mut actual_length: GLint = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(max_length as usize);
    unsafe {
        gl::GetProgramInfoLog(program, max_length, &mut actual_length, buf.as_mut_ptr() as *mut _);
        buf.set_len(actual_length as usize);
    }

    String::from_utf8_lossy(&buf).to_string()
}

fn get_shader_info_log(shader: GLuint) -> String {
    let mut max_length: GLint = 0;
    unsafe {
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut max_length);
    } 

    let mut actual_length: GLint = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(max_length as usize);
    unsafe {
        gl::GetShaderInfoLog(shader, max_length, &mut actual_length, buf.as_mut_ptr() as *mut _);
        buf.set_len(actual_length as usize);
    }

    String::from_utf8_lossy(&buf).to_string()
}

#[derive(Debug)]
pub enum ShaderError {
    Compile(String),
    Link(String),
    Uniform(&'static CStr),
}

impl std::error::Error for ShaderError {}

impl fmt::Display for ShaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compile(reason) => 
                write!(f, "Failed compiling shader: {}", reason),
            Self::Link(reason) => 
                write!(f, "Failed linking shader: {}", reason),
            Self::Uniform(name) => 
                write!(f, "Failed to get uniform location of {:?}", name),
        }
    }
}


