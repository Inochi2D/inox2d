#![allow(dead_code)]

mod buffers;
mod node_bundle;
mod pipeline;

use crate::nodes::node::InoxNodeUuid;
use crate::nodes::node_data::InoxData;
use crate::nodes::node_tree::InoxNodeTree;
use crate::puppet::Puppet;
use crate::render::RenderCtxKind;
use crate::texture::decode_model_textures;
use crate::{model::Model, nodes::node_data::MaskMode};

use encase::ShaderType;
use glam::{Vec2, Vec3};
use tracing::warn;
use wgpu::{util::DeviceExt, *};

use self::node_bundle::{CompositeData, PartData};
use self::pipeline::CameraData;
use self::{
    buffers::buffers_for_puppet,
    node_bundle::{node_bundles_for_model, NodeBundle},
    pipeline::{InoxPipeline, Uniform},
};

pub struct Renderer {
    setup: InoxPipeline,
    composite_texture: Option<Texture>,
    model_texture_binds: Vec<BindGroup>,
    buffers: buffers::InoxBuffers,
    bundles: Vec<node_bundle::NodeBundle>,
}

impl Renderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        texture_format: TextureFormat,
        model: &Model,
    ) -> Self {
        let setup = InoxPipeline::create(device, texture_format);

        let mut model_texture_binds = Vec::new();

        let sampler = device.create_sampler(&SamplerDescriptor {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToBorder,
            address_mode_v: AddressMode::ClampToBorder,
            border_color: Some(SamplerBorderColor::TransparentBlack),
            ..SamplerDescriptor::default()
        });

        let shalltexs = decode_model_textures(&model.textures);
        for shalltex in &shalltexs {
            let texture_size = wgpu::Extent3d {
                width: shalltex.width(),
                height: shalltex.height(),
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    label: Some("texture"),
                    view_formats: &[],
                },
                shalltex.pixels(),
            );

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let texture_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &setup.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("texture bind group"),
            });
            model_texture_binds.push(texture_bind);
        }

        let buffers = buffers_for_puppet(device, &model.puppet, setup.uniform_alignment_needed);
        let bundles = node_bundles_for_model(
            device,
            &setup,
            &buffers,
            &model_texture_binds,
            &model.puppet,
        );

        Self {
            setup,
            buffers,
            bundles,

            composite_texture: None,
            model_texture_binds,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_part(
        &self,
        puppet: &Puppet,

        view: &TextureView,
        mask_view: &TextureView,
        uniform_group: &BindGroup,

        op: LoadOp<Color>,
        encoder: &mut CommandEncoder,

        PartData(bundle, masks): &PartData,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Part Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: op,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: mask_view,
                depth_ops: None,
                stencil_ops: Some(Operations {
                    load: wgpu::LoadOp::Clear(u32::from(!masks.is_empty())),
                    store: true,
                }),
            }),
        });

        render_pass.set_vertex_buffer(0, self.buffers.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.buffers.uv_buffer.slice(..));
        render_pass.set_vertex_buffer(2, self.buffers.deform_buffer.slice(..));
        render_pass.set_index_buffer(
            self.buffers.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.set_pipeline(&self.setup.mask_pipeline);

        for mask in masks {
            let node = puppet.nodes.get_node(mask.source).unwrap();
            let part = if let InoxData::Part(part) = &node.data {
                part
            } else {
                todo!()
            };

            render_pass.set_bind_group(1, &self.model_texture_binds[part.tex_albedo], &[]);
            render_pass.set_bind_group(2, &self.model_texture_binds[part.tex_emissive], &[]);
            render_pass.set_bind_group(3, &self.model_texture_binds[part.tex_bumpmap], &[]);

            render_pass.set_bind_group(
                0,
                uniform_group,
                &[(self.setup.uniform_alignment_needed
                    * self.buffers.uniform_index_map[&mask.source]) as u32],
            );

            match mask.mode {
                MaskMode::Mask => {
                    render_pass.set_stencil_reference(0);
                }
                MaskMode::Dodge => {
                    render_pass.set_stencil_reference(1);
                }
            }

            let node_rinf = &puppet.render_ctx.node_render_ctxs[&mask.source];
            if let RenderCtxKind::Part(pinf) = &node_rinf.kind {
                let range =
                    (pinf.index_offset as u32)..(pinf.index_offset as u32 + pinf.index_len as u32);
                render_pass.draw_indexed(range, 0, 0..1);
            } else {
                warn!(
                    "Node mask {:?} is not a part but is trying to get rendered as one",
                    mask.source
                );
            }
        }

        render_pass.set_stencil_reference(0);
        render_pass.execute_bundles(std::iter::once(bundle));

        drop(render_pass);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_composite(
        &self,
        puppet: &Puppet,

        view: &TextureView,
        mask_view: &TextureView,
        uniform_group: &BindGroup,

        op: LoadOp<Color>,
        encoder: &mut CommandEncoder,

        composite_view: &TextureView,
        composite_bind: &BindGroup,
        CompositeData(parts, uuid): &CompositeData,
    ) {
        for data in parts {
            self.render_part(
                puppet,
                composite_view,
                mask_view,
                uniform_group,
                op,
                encoder,
                data,
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: op,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: mask_view,
                depth_ops: None,
                stencil_ops: None,
            }),
        });

        render_pass.set_vertex_buffer(0, self.buffers.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.buffers.uv_buffer.slice(..));
        render_pass.set_vertex_buffer(2, self.buffers.deform_buffer.slice(..));
        render_pass.set_index_buffer(
            self.buffers.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        let child = puppet.nodes.get_node(*uuid).unwrap();
        let child = if let InoxData::Composite(comp) = &child.data {
            comp
        } else {
            todo!()
        };

        render_pass.set_pipeline(&self.setup.composite_pipelines[&child.draw_state.blend_mode]);

        render_pass.set_bind_group(
            0,
            uniform_group,
            &[(self.setup.uniform_alignment_needed * self.buffers.uniform_index_map[uuid]) as u32],
        );
        render_pass.set_bind_group(1, composite_bind, &[]);
        render_pass.set_bind_group(2, composite_bind, &[]);
        render_pass.set_bind_group(3, composite_bind, &[]);
        render_pass.draw_indexed(0..6, 0, 0..1);

        drop(render_pass);
    }

    /// It is a logical error to pass in a different puppet than the one passed to create.
    pub fn render(&mut self, queue: &Queue, device: &Device, puppet: &Puppet, view: &TextureView) {
        let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("inox2d uniform bind group"),
            layout: &self.setup.uniform_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffers.camera_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(CameraData::min_size().get()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.buffers.uniform_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(Uniform::min_size().into()),
                    }),
                },
            ],
        });

        let composite_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("texture"),
            view_formats: &[],
        });

        let composite_view = composite_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mask_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("texture"),
            view_formats: &[],
        });

        let mask_view = mask_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToBorder,
            address_mode_v: AddressMode::ClampToBorder,
            border_color: Some(SamplerBorderColor::TransparentBlack),
            ..SamplerDescriptor::default()
        });

        let composite_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.setup.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&composite_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture bind group"),
        });

        for uuid in puppet.nodes.all_node_ids() {
            let node = puppet.nodes.get_node(uuid).unwrap();

            let unif = match &node.data {
                InoxData::Part(_) => Uniform {
                    opacity: 1.0,
                    mult_color: Vec3::ONE,
                    screen_color: Vec3::ZERO,
                    emission_strength: 0.0,
                    offset: puppet.render_ctx.node_render_ctxs[&uuid]
                        .trans_offset
                        .translation
                        .truncate(),
                },
                InoxData::Composite(_) => Uniform {
                    opacity: 1.0,
                    mult_color: Vec3::ONE,
                    screen_color: Vec3::ZERO,
                    emission_strength: 0.0,
                    offset: Vec2::ZERO,
                },
                _ => continue,
            };

            let mut buffer = encase::UniformBuffer::new(Vec::new());
            buffer.write(&unif).unwrap();
            queue.write_buffer(
                &self.buffers.uniform_buffer,
                (self.setup.uniform_alignment_needed * self.buffers.uniform_index_map[&uuid])
                    as u64,
                buffer.as_ref(),
            );
        }

        let mut first = true;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Part Render Encoder"),
        });

        for bundle in &self.bundles {
            let op = if first {
                first = false;

                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
            } else {
                wgpu::LoadOp::Load
            };

            match bundle {
                NodeBundle::Part(data) => {
                    self.render_part(
                        puppet,
                        view,
                        &mask_view,
                        &uniform_group,
                        op,
                        &mut encoder,
                        data,
                    );
                }
                NodeBundle::Composite(data) => {
                    self.render_composite(
                        puppet,
                        view,
                        &mask_view,
                        &uniform_group,
                        op,
                        &mut encoder,
                        &composite_view,
                        &composite_bind,
                        data,
                    );
                }
            }
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
}

fn node_absolute_translation<T>(nodes: &InoxNodeTree<T>, uuid: InoxNodeUuid) -> Vec3 {
    nodes
        .ancestors(uuid)
        .filter_map(|n| nodes.arena.get(n))
        .map(|n| n.get().trans_offset.translation)
        .sum::<Vec3>()
}
