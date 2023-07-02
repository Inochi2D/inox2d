use std::collections::HashMap;

use encase::ShaderType;
use glam::{Vec2, Vec3};
use wgpu::*;

use crate::nodes::node_data::BlendMode;

fn blend_state_for_blend_mode(mode: BlendMode) -> BlendState {
    let component = match mode {
        BlendMode::Normal => BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        BlendMode::Multiply => BlendComponent {
            src_factor: BlendFactor::Dst,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        BlendMode::ColorDodge => BlendComponent {
            src_factor: BlendFactor::Dst,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        BlendMode::LinearDodge => BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        BlendMode::Screen => BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::OneMinusSrc,
            operation: BlendOperation::Add,
        },
        BlendMode::ClipToLower => BlendComponent {
            src_factor: BlendFactor::DstAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        },
        BlendMode::SliceFromLower => BlendComponent {
            src_factor: BlendFactor::DstAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Subtract,
        },
    };

    BlendState {
        color: component,
        alpha: component,
    }
}

#[allow(clippy::too_many_arguments)]
fn create_part_pipeline(
    device: &Device,
    label: Label<'_>,
    layout: &PipelineLayout,
    texture_format: TextureFormat,
    fragment: &ShaderModule,
    vertex: &ShaderModule,
    composite: bool,
    blend: BlendState,
) -> RenderPipeline {
    let face_state = StencilFaceState {
        compare: if composite {
            CompareFunction::Always
        } else {
            CompareFunction::Equal
        },
        ..StencilFaceState::default()
    };
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        fragment: Some(FragmentState {
            module: fragment,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: texture_format,
                blend: Some(blend),
                write_mask: ColorWrites::ALL,
            })],
        }),
        vertex: VertexState {
            module: vertex,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                },
            ],
        },
        primitive: PrimitiveState {
            cull_mode: None,
            ..PrimitiveState::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil: StencilState {
                front: face_state,
                back: face_state,
                read_mask: 0xff,
                write_mask: if composite { 0 } else { 0xff },
            },
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        multiview: None,
    })
}

fn create_stencil_pipeline(
    device: &Device,
    label: Label<'_>,
    layout: &PipelineLayout,
    texture_format: TextureFormat,
    fragment: &ShaderModule,
    vertex: &ShaderModule,
) -> RenderPipeline {
    let face_state = StencilFaceState {
        compare: CompareFunction::Always,
        fail_op: StencilOperation::Keep,
        depth_fail_op: StencilOperation::Keep,
        pass_op: StencilOperation::Replace,
    };

    device.create_render_pipeline(&RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        fragment: Some(FragmentState {
            module: fragment,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: texture_format,
                blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: ColorWrites::empty(),
            })],
        }),
        vertex: VertexState {
            module: vertex,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                },
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                },
            ],
        },
        primitive: PrimitiveState {
            cull_mode: None,
            ..PrimitiveState::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: CompareFunction::Always,
            stencil: StencilState {
                front: face_state,
                back: face_state,
                read_mask: 0xff,
                write_mask: 0xff,
            },
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        multiview: None,
    })
}

#[derive(Debug)]
pub struct InoxPipeline {
    pub basic_pipelines: HashMap<BlendMode, RenderPipeline>,
    pub composite_pipelines: HashMap<BlendMode, RenderPipeline>,
    pub mask_pipeline: RenderPipeline,

    pub uniform_layout: BindGroupLayout,
    pub texture_layout: BindGroupLayout,
    pub texture_format: TextureFormat,
    pub uniform_alignment_needed: usize,
}

impl InoxPipeline {
    pub fn create(device: &Device, texture_format: TextureFormat) -> Self {
        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("inox2d uniform bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(Uniform::min_size().get()),
                    },
                    count: None,
                },
            ],
        });

        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("inox2d texture bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("inox2d basic pipeline layout"),
            bind_group_layouts: &[
                &uniform_layout,
                &texture_layout,
                &texture_layout,
                &texture_layout,
            ],
            push_constant_ranges: &[],
        });

        let mask_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("inox2d mask pipeline layout"),
            bind_group_layouts: &[&uniform_layout, &texture_layout],
            push_constant_ranges: &[],
        });

        // for (mode, state) in get

        let mut basic_pipelines: HashMap<BlendMode, RenderPipeline> = HashMap::new();
        for mode in BlendMode::VALUES {
            let basic_pipeline = create_part_pipeline(
                device,
                Some("inox2d basic pipeline"),
                &pipeline_layout,
                texture_format,
                &device.create_shader_module(include_wgsl!("shaders/basic/basic.frag.wgsl")),
                &device.create_shader_module(include_wgsl!("shaders/basic/basic.vert.wgsl")),
                false,
                blend_state_for_blend_mode(mode),
            );

            basic_pipelines.insert(mode, basic_pipeline);
        }

        let mut composite_pipelines: HashMap<BlendMode, RenderPipeline> = HashMap::new();
        for mode in BlendMode::VALUES {
            let composite_pipeline = create_part_pipeline(
                device,
                Some("inox2d composite pipeline"),
                &pipeline_layout,
                texture_format,
                &device.create_shader_module(include_wgsl!("shaders/basic/composite.frag.wgsl")),
                &device.create_shader_module(include_wgsl!("shaders/basic/composite.vert.wgsl")),
                true,
                blend_state_for_blend_mode(mode),
            );

            composite_pipelines.insert(mode, composite_pipeline);
        }

        let mask_pipeline = create_stencil_pipeline(
            device,
            Some("inox2d mask pipeline"),
            &mask_pipeline_layout,
            texture_format,
            &device.create_shader_module(include_wgsl!("shaders/basic/mask.frag.wgsl")),
            &device.create_shader_module(include_wgsl!("shaders/basic/mask.vert.wgsl")),
        );

        let min_uniform_buffer_offset_alignment =
            device.limits().min_uniform_buffer_offset_alignment;

        InoxPipeline {
            basic_pipelines,
            composite_pipelines,
            mask_pipeline,

            uniform_layout,
            texture_layout,
            texture_format,
            uniform_alignment_needed: (Uniform::min_size().get())
                .max(min_uniform_buffer_offset_alignment.into())
                as usize,
        }
    }
}

#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
pub struct Uniform {
    pub opacity: f32,
    pub mult_color: Vec3,
    pub screen_color: Vec3,
    pub emission_strength: f32,
    pub offset: Vec2,
}
