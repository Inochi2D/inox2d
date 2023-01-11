use std::ops::Deref;

use glam::{Mat4, Vec2, Vec3};
use glow::HasContext;

use super::shader::{self, ShaderCompileError};

#[derive(Clone)]
pub struct PartShader {
    program: glow::NativeProgram,
    u_mvp: Option<glow::NativeUniformLocation>,
    u_offset: Option<glow::NativeUniformLocation>,
    u_opacity: Option<glow::NativeUniformLocation>,
    u_mult_color: Option<glow::NativeUniformLocation>,
    u_screen_color: Option<glow::NativeUniformLocation>,
}

impl Deref for PartShader {
    type Target = glow::NativeProgram;

    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

impl PartShader {
    const PART_VERT: &str = include_str!("shaders/basic/basic.vert");
    const PART_FRAG: &str = include_str!("shaders/basic/basic.frag");

    pub fn new(gl: &glow::Context) -> Result<Self, ShaderCompileError> {
        let program = shader::compile(gl, Self::PART_VERT, Self::PART_FRAG)?;

        Ok(Self {
            program,
            u_mvp: unsafe { gl.get_uniform_location(program, "mvp") },
            u_offset: unsafe { gl.get_uniform_location(program, "offset") },
            u_opacity: unsafe { gl.get_uniform_location(program, "opacity") },
            u_mult_color: unsafe { gl.get_uniform_location(program, "multColor") },
            u_screen_color: unsafe { gl.get_uniform_location(program, "screenColor") },
        })
    }

    /// Binds the shader to OpenGL.
    ///
    /// # Safety
    ///
    /// This calls `glUseProgram`, apply your OpenGL knowledge to know if it is safe.
    #[inline]
    pub unsafe fn bind(&self, gl: &glow::Context) {
        gl.use_program(Some(self.program));
    }

    /// Sets the `mvp` uniform of the shader.
    ///
    /// # Safety
    ///
    /// The uniform exists and the type used to set it is valid as well.
    #[inline]
    pub unsafe fn set_mvp(&self, gl: &glow::Context, mvp: Mat4) {
        gl.uniform_matrix_4_f32_slice(self.u_mvp.as_ref(), false, mvp.as_ref());
    }

    /// Sets the `offset` uniform of the shader.
    ///
    /// # Safety
    ///
    /// The uniform exists and the type used to set it is valid as well.
    #[inline]
    pub unsafe fn set_offset(&self, gl: &glow::Context, offset: Vec2) {
        gl.uniform_2_f32_slice(self.u_offset.as_ref(), offset.as_ref());
    }

    /// Sets the `opacity` uniform of the shader.
    ///
    /// # Safety
    ///
    /// The uniform exists and the type used to set it is valid as well.
    #[inline]
    pub unsafe fn set_opacity(&self, gl: &glow::Context, opacity: f32) {
        gl.uniform_1_f32(self.u_opacity.as_ref(), opacity);
    }

    /// Sets the `multColor` uniform of the shader.
    ///
    /// # Safety
    ///
    /// The uniform exists and the type used to set it is valid as well.
    #[inline]
    pub unsafe fn set_mult_color(&self, gl: &glow::Context, mult_color: Vec3) {
        gl.uniform_3_f32_slice(self.u_mult_color.as_ref(), mult_color.as_ref());
    }

    /// Sets the `screenColor` uniform of the shader.
    ///
    /// # Safety
    ///
    /// The uniform exists and the type used to set it is valid as well.
    #[inline]
    pub unsafe fn set_screen_color(&self, gl: &glow::Context, screen_color: Vec3) {
        gl.uniform_3_f32_slice(self.u_screen_color.as_ref(), screen_color.as_ref());
    }
}
