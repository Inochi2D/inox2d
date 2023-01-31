use encase::ShaderType;
use wgpu::{BindGroup, Device, RenderBundle};

use crate::{
    nodes::{
        node::InoxNodeUuid,
        node_data::{InoxData, Mask, Part},
    },
    puppet::Puppet,
};

use super::{
    buffers::InoxBuffers,
    pipeline::{CameraData, InoxPipeline, Uniform},
};

#[derive(Debug)]
pub struct PartData(pub RenderBundle, pub Vec<Mask>);

#[derive(Debug)]
pub enum NodeBundle {
    Part(PartData),
    Composite(Vec<PartData>, InoxNodeUuid),
}

fn part_bundle_for_part(
    device: &Device,
    setup: &InoxPipeline,
    buffers: &InoxBuffers,
    model_texture_binds: &[BindGroup],
    uniform_group: &BindGroup,

    uuid: InoxNodeUuid,
    part: &Part,
) -> PartData {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some(&format!("part encoder: {:?}", uuid)),
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
        uniform_group,
        &[(setup.uniform_alignment_needed * buffers.uniform_index_map[&uuid]) as u32],
    );
    encoder.set_bind_group(1, &model_texture_binds[part.tex_albedo], &[]);
    encoder.set_bind_group(2, &model_texture_binds[part.tex_emissive], &[]);
    encoder.set_bind_group(3, &model_texture_binds[part.tex_bumpmap], &[]);

    let range = buffers.part_index_map[&uuid].clone();
    encoder.draw_indexed(range, 0, 0..1);

    let bundle = encoder.finish(&wgpu::RenderBundleDescriptor {
        label: Some(&format!("part bundle: {:?}", uuid)),
    });

    PartData(bundle, part.draw_state.masks.clone())
}

pub fn node_bundles_for_model(
    device: &Device,
    setup: &InoxPipeline,
    buffers: &InoxBuffers,
    model_texture_binds: &[BindGroup],

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

    for uuid in puppet.nodes.zsorted_root() {
        let node = puppet.nodes.get_node(uuid).unwrap();

        if let InoxData::Part(part) = &node.data {
            out.push(NodeBundle::Part(part_bundle_for_part(
                device,
                setup,
                buffers,
                model_texture_binds,
                &uniform_group,
                uuid,
                part,
            )));
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

            let mut bundles = Vec::new();

            for child_id in puppet.nodes.zsorted_child(uuid) {
                if child_id == uuid {
                    continue;
                }
                let child = puppet.nodes.get_node(child_id).unwrap();

                if let InoxData::Part(part) = &child.data {
                    let bundle = part_bundle_for_part(
                        device,
                        setup,
                        buffers,
                        model_texture_binds,
                        &uniform_group,
                        child_id,
                        part,
                    );
                    bundles.push(bundle);
                }
            }

            out.push(NodeBundle::Composite(bundles, uuid));
        }
    }

    out
}