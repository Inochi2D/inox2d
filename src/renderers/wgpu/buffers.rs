use std::{collections::HashMap, ops::Range};

use encase::ShaderType;
use glam::Vec2;
use wgpu::{util::DeviceExt, Buffer, BufferDescriptor, BufferUsages, Device};

use crate::{
    nodes::{node::InoxNodeUuid, node_data::InoxData},
    puppet::Puppet,
};

use super::pipeline::CameraData;

pub struct InoxBuffers {
    pub camera_buffer: Buffer,
    pub uniform_buffer: Buffer,
    pub uniform_index_map: HashMap<InoxNodeUuid, u64>,
    pub part_index_map: HashMap<InoxNodeUuid, Range<u32>>,

    pub vertex_buffer: Buffer,
    pub uv_buffer: Buffer,
    pub deform_buffer: Buffer,
    pub index_buffer: Buffer,
}

pub fn buffers_for_puppet(
    device: &Device,
    puppet: &Puppet,
    uniform_alignment_needed: u64,
) -> InoxBuffers {
    let mut parts = 0;

    let mut verts: Vec<Vec2> = vec![
        Vec2::new(-1.0, -1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(1.0, -1.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, 1.0),
    ];
    let mut uvs: Vec<Vec2> = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
    ];
    let mut deforms: Vec<Vec2> = vec![Vec2::ZERO; 6];

    let mut indexes: Vec<u16> = Vec::new();

    let mut uniform_index_map: HashMap<InoxNodeUuid, u64> = HashMap::new();
    let mut part_index_map: HashMap<InoxNodeUuid, Range<u32>> = HashMap::new();

    for node in puppet.nodes.arena.iter() {
        match &node.get().data {
            InoxData::Part(part) => {
                let mesh = &part.mesh;
                let offset = verts.len() as u16;

                let num_verts = mesh.vertices.len();
                assert_eq!(num_verts, mesh.uvs.len());

                verts.extend_from_slice(&mesh.vertices);
                uvs.extend_from_slice(&mesh.uvs);
                deforms.resize(deforms.len() + num_verts, Vec2::ZERO);

                let start_len = indexes.len() as u32;
                let end_len = (indexes.len() + mesh.indices.len()) as u32;
                part_index_map.insert(node.get().uuid, start_len..end_len);
                indexes.extend(mesh.indices.iter().map(|index| index + offset));
            }
            InoxData::Composite(_) => {}
            _ => continue,
        }

        uniform_index_map.insert(node.get().uuid, parts);
        parts += 1;
    }

    let camera_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("inox2d uniform buffer"),
        size: CameraData::min_size().get(),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("inox2d uniform buffer"),
        size: uniform_alignment_needed * parts,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertex buffer"),
        contents: bytemuck::cast_slice(&verts),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let uv_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("uv buffer"),
        contents: bytemuck::cast_slice(&uvs),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let deform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("deform buffer"),
        contents: bytemuck::cast_slice(&deforms),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("index buffer"),
        contents: bytemuck::cast_slice(&indexes),
        usage: wgpu::BufferUsages::INDEX,
    });

    InoxBuffers {
        camera_buffer,
        uniform_buffer,
        uniform_index_map,
        part_index_map,

        vertex_buffer,
        uv_buffer,
        deform_buffer,
        index_buffer,
    }
}
