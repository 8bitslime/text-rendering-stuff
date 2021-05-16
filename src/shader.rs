use gl::types::*;
use std::io::{Error, ErrorKind, Read, Result};
use std::fs::File;
use std::path::Path;

pub enum Stage {
    Vertex,
    Fragment
}
impl Stage {
    pub(crate) fn opengl_enum(&self) -> GLenum {
        match self {
            Self::Vertex => gl::VERTEX_SHADER,
            Self::Fragment => gl::FRAGMENT_SHADER
        }
    }
}

pub struct Shader(pub GLuint);
impl Shader {
    pub fn from_bytes(bytes: &[u8], stage: Stage) -> Result<Self> {
        let data   = [bytes.as_ptr() as *const i8];
        let length = [bytes.len() as i32];
        
        
        let shader = unsafe {
            let shader = gl::CreateShader(stage.opengl_enum());
            gl::ShaderSource(shader, 1, data.as_ptr(), length.as_ptr());
            gl::CompileShader(shader);
            
            let mut status: GLint = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
            if status == 0 {
                let mut length: GLint = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length);
                
                let mut buffer = vec![0i8; length as usize];
                gl::GetShaderInfoLog(shader, length, &mut length, buffer.as_mut_ptr());
                let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr());
                
                gl::DeleteShader(shader);
                Err(Error::new(ErrorKind::InvalidInput, c_str.to_str().unwrap().to_owned()))
            } else {
                Ok(shader)
            }
            
        };
        
        Ok(Self(shader?))
    }
    
    pub fn from_string(string: &str, stage: Stage) -> Result<Self> {
        Shader::from_bytes(string.as_bytes(), stage)
    }
    
    pub fn from_file<P: AsRef<Path>>(path: P, stage: Stage) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::<u8>::with_capacity(512);
        file.read_to_end(&mut buffer)?;
        Shader::from_bytes(&buffer, stage)
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.0);
        }
    }
}

pub struct Program(pub GLuint);
impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Self> {
        let program = unsafe {
            let program = gl::CreateProgram();
            
            for shader in shaders {
                gl::AttachShader(program, shader.0);
            }
            gl::LinkProgram(program);
            for shader in shaders {
                gl::DetachShader(program, shader.0);
            }
            
            let mut status: GLint = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            if status == 0 {
                let mut length: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length);
                
                let mut buffer = vec![0i8; length as usize];
                gl::GetProgramInfoLog(program, length, &mut length, buffer.as_mut_ptr());
                let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr());
                
                gl::DeleteProgram(program);
                Err(Error::new(ErrorKind::InvalidInput, c_str.to_str().unwrap().to_owned()))
            } else {
                Ok(program)
            }
        };
        
        Ok(Self(program?))
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}
