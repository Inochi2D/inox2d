use gl::types::{GLint, GLuint};

use crate::core::shader::Shader;
use crate::core::texture::Texture;

struct PartGlobalState {
    bound_albedo: Texture,

    part_shader: Shader,
    part_mask_shader: Shader,

    /* GLSL Uniforms (Normal) */
    mvp: GLint,
    offset: GLint,
    g_opacity: GLint,
    g_mult_color: GLint,
    g_screen_color: GLint,
    g_emission_strength: GLint,

    /* GLSL Uniforms (Masks) */
    m_mvp: GLint,
    m_threshold: GLint,

    s_vertex_buffer: GLuint,
    s_uv_buffer: GLuint,
    s_element_buffer: GLuint,
}