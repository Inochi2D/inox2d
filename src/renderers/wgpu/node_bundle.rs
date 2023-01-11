use encase::ShaderType;
use wgpu::{BindGroup, Device, RenderBundle};

use crate::{
    nodes::{
        node::InoxNodeUuid,
        node_data::{Drawable, InoxData},
    },
    puppet::Puppet,
};

use super::{
    buffers::InoxBuffers,
    pipeline::{CameraData, InoxPipeline, Uniform},
};

pub enum NodeBundle {
    Part(RenderBundle, Drawable),
    Composite(RenderBundle, InoxNodeUuid),
}
pub fn node_bundles_for_model(
    device: &Device,
    setup: &InoxPipeline,
    buffers: &InoxBuffers,
    model_texture_binds: &Vec<BindGroup>,

    puppet: &Puppet,
) -> Vec<NodeBundle> {
    let uniform_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("inox2d uniform bind group"),
        layout: &setup.uniform_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffers.camera_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(CameraData::min_size().get()),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffers.uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(Uniform::min_size().get()),
                }),
            },
        ],
    });

    let mut out = Vec::new();

    for uuid in puppet.nodes.zsorted() {
        let node = puppet.nodes.get_node(uuid).unwrap();

        if let InoxData::Part(part) = &node.data {
            let mut encoder =
                device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some(&format!("regular node encoder: {:?}", uuid)),
                    color_formats: &[Some(setup.texture_format)],
                    depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_read_only: true,
                        stencil_read_only: false,
                    }),
                    sample_count: 1,
                    ..Default::default()
                });

            encoder.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, buffers.uv_buffer.slice(..));
            encoder.set_vertex_buffer(2, buffers.deform_buffer.slice(..));
            encoder.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            encoder.set_pipeline(&setup.basic_pipelines[&part.draw_state.blend_mode]);

            encoder.set_bind_group(
                0,
                &uniform_group,
                &[(setup.uniform_alignment_needed * buffers.uniform_index_map[&uuid]) as u32],
            );
            encoder.set_bind_group(1, &model_texture_binds[part.tex_albedo], &[]);
            encoder.set_bind_group(2, &model_texture_binds[part.tex_emissive], &[]);
            encoder.set_bind_group(3, &model_texture_binds[part.tex_bumpmap], &[]);

            let range = buffers.part_index_map[&uuid].clone();
            encoder.draw_indexed(range, 0, 0..1);

            let bundle = encoder.finish(&wgpu::RenderBundleDescriptor {
                label: Some(&format!("regular node bundle: {:?}", uuid)),
            });

            out.push(NodeBundle::Part(bundle, part.draw_state.clone()));
        } else if let InoxData::Composite(_) = &node.data {
            let mut encoder =
                device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some(&format!("composite children encoder: {:?}", uuid)),
                    color_formats: &[Some(setup.texture_format)],
                    depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                        format: wgpu::TextureFormat::Depth24PlusStencil8,
                        depth_read_only: true,
                        stencil_read_only: false,
                    }),
                    sample_count: 1,
                    ..Default::default()
                });

            encoder.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            encoder.set_vertex_buffer(1, buffers.uv_buffer.slice(..));
            encoder.set_vertex_buffer(2, buffers.deform_buffer.slice(..));
            encoder.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            for child_id in puppet.nodes.get_children_uuids(uuid).unwrap_or_default() {
                let child = puppet.nodes.get_node(child_id).unwrap();

                if let InoxData::Part(part) = &child.data {
                    encoder.set_pipeline(&setup.basic_pipelines[&part.draw_state.blend_mode]);

                    encoder.set_bind_group(
                        0,
                        &uniform_group,
                        &[
                            (setup.uniform_alignment_needed * buffers.uniform_index_map[&child_id])
                                as u32,
                        ],
                    );
                    encoder.set_bind_group(1, &model_texture_binds[part.tex_albedo], &[]);
                    encoder.set_bind_group(2, &model_texture_binds[part.tex_emissive], &[]);
                    encoder.set_bind_group(3, &model_texture_binds[part.tex_bumpmap], &[]);

                    let range = buffers.part_index_map[&child_id].clone();
                    encoder.draw_indexed(range, 0, 0..1);
                }
            }

            let bundle = encoder.finish(&wgpu::RenderBundleDescriptor {
                label: Some(&format!("composite children bundle: {:?}", uuid)),
            });

            out.push(NodeBundle::Composite(bundle, uuid));
        }
    }

    out
}
