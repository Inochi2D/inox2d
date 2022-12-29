use glow::HasContext;
use wgpu::{Device, RenderPipeline, SurfaceConfiguration};

#[derive(thiserror::Error, Debug)]
#[error("Could not compile shader: {0}")]
pub struct ShaderCompileError(String);

/// Compiles a shader program composed of a vertex and fragment shader.
pub(crate) fn compile(
    device: &Device,
    config: &SurfaceConfiguration,
    vertex: &str,
    fragment: &str,
) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(vertex.into()).into(),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main", // 1.
            buffers: &[],           // 2.
        },
        fragment: Some(wgpu::FragmentState {
            // 3.
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                // 4.
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        // continued ...
        depth_stencil: None, // 1.
        multisample: wgpu::MultisampleState {
            count: 1,                         // 2.
            mask: !0,                         // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview: None, // 5.
    })
}

unsafe fn verify_shader(
    gl: &glow::Context,
    shader: glow::NativeShader,
) -> Result<(), ShaderCompileError> {
    if gl.get_shader_compile_status(shader) {
        Ok(())
    } else {
        Err(ShaderCompileError(gl.get_shader_info_log(shader)))
    }
}

unsafe fn verify_program(
    gl: &glow::Context,
    program: glow::NativeProgram,
) -> Result<(), ShaderCompileError> {
    if gl.get_program_link_status(program) {
        Ok(())
    } else {
        Err(ShaderCompileError(gl.get_program_info_log(program)))
    }
}
