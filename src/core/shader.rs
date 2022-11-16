use std::ffi::{CStr, CString};
use std::ptr;

use gl::types::*;
use glam::{Vec2, Vec3, Vec4};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Shader error: {0}")]
pub struct ShaderError(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shader {
    shader_program: GLuint,
    frag_shader: GLuint,
    vert_shader: GLuint,
}

fn verify_shader(shader: GLuint) -> Result<(), ShaderError> {
    let mut compile_status: GLint = gl::FALSE as GLint;
    unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status) };
    if compile_status == gl::FALSE as GLint {
        // Get the length of the error log
        let mut log_len: GLint = 0;
        unsafe { gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len) };

        let mut log = vec![0 as GLchar; log_len as usize];
        let cstr_log = unsafe {
            gl::GetShaderInfoLog(shader, log_len, ptr::null_mut(), log.as_mut_ptr());
            CString::from_raw(log.as_mut_ptr())
        };

        Err(ShaderError(cstr_log.into_string().unwrap()))
    } else {
        Ok(())
    }
}

fn verify_program(program: GLuint) -> Result<(), ShaderError> {
    let mut link_status = gl::FALSE as GLint;
    unsafe { gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status) };
    if link_status == gl::FALSE as GLint {
        // Get the length of the error log
        let mut log_len: GLint = 0;
        unsafe { gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len) };

        let mut log = vec![0 as GLchar; log_len as usize];
        let cstr_log = unsafe {
            gl::GetProgramInfoLog(program, log_len, ptr::null_mut(), log.as_mut_ptr());
            CString::from_raw(log.as_mut_ptr())
        };

        Err(ShaderError(cstr_log.into_string().unwrap()))
    } else {
        Ok(())
    }
}

impl Shader {
    /// Compiles a new shader from source.
    pub fn new(vertex: &str, fragment: &str) -> Result<Shader, ShaderError> {
        // Compile vertex shader
        let c_vert = CString::new(vertex).unwrap();
        let vert_shader = unsafe {
            let shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(shader, 1, &c_vert.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            verify_shader(shader)?;
            shader
        };

        // Compile fragment shader
        let c_frag = CString::new(fragment).unwrap();
        let frag_shader = unsafe {
            let shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(shader, 1, &c_frag.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            verify_shader(shader)?;
            shader
        };

        // Attach and link them
        let shader_program = unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vert_shader);
            gl::AttachShader(program, frag_shader);
            gl::LinkProgram(program);
            verify_program(program)?;
            program
        };

        Ok(Shader {
            shader_program,
            frag_shader,
            vert_shader,
        })
    }

    /// Use the shader.
    pub fn use_program(&self) {
        unsafe { gl::UseProgram(self.shader_program) };
    }

    pub fn get_uniform_location(&self, name: &CStr) -> GLint {
        unsafe { gl::GetUniformLocation(self.shader_program, name.as_ptr()) }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DetachShader(self.shader_program, self.vert_shader);
            gl::DetachShader(self.shader_program, self.frag_shader);
            gl::DeleteProgram(self.shader_program);

            gl::DeleteShader(self.frag_shader);
            gl::DeleteShader(self.vert_shader);
        }
    }
}

pub fn set_uniform_bool(uniform: GLint, value: bool) {
    unsafe { gl::Uniform1i(uniform, value as GLint) };
}

pub fn set_uniform_int(uniform: GLint, value: i32) {
    unsafe { gl::Uniform1i(uniform, value as GLint) };
}

pub fn set_uniform_float(uniform: GLint, value: f32) {
    unsafe { gl::Uniform1f(uniform, value as GLfloat) };
}

pub fn set_uniform_vec2(uniform: GLint, value: Vec2) {
    unsafe { gl::Uniform2f(uniform, value.x, value.y) };
}

pub fn set_uniform_vec3(uniform: GLint, value: Vec3) {
    unsafe { gl::Uniform3f(uniform, value.x, value.y, value.z) };
}

pub fn set_uniform_vec4(uniform: GLint, value: Vec4) {
    unsafe { gl::Uniform4f(uniform, value.x, value.y, value.z, value.w) };
}

pub fn set_uniform_mat4(uniform: GLint, value: &[f32; 16]) {
    unsafe { gl::UniformMatrix4fv(uniform, 1, gl::TRUE, value.as_ptr()) }
}
