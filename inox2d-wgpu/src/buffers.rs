use std::collections::HashMap;

use wgpu::{util::DeviceExt, Buffer, BufferDescriptor, BufferUsages, Device};

use inox2d::{nodes::node::InoxNodeUuid, puppet::Puppet};

pub struct InoxBuffers {
	pub uniform_buffer: Buffer,
	pub uniform_index_map: HashMap<InoxNodeUuid, usize>,

	pub vertex_buffer: Buffer,
	pub uv_buffer: Buffer,
	pub deform_buffer: Buffer,
	pub index_buffer: Buffer,
}

pub fn buffers_for_puppet(device: &Device, puppet: &Puppet, uniform_alignment_needed: usize) -> InoxBuffers {
	let mut uniform_index_map: HashMap<InoxNodeUuid, usize> = HashMap::new();

	for (i, node) in (puppet.nodes.arena.iter())
		.map(|arena_node| arena_node.get())
		.filter(|node| node.is_part() || node.is_composite())
		.enumerate()
	{
		uniform_index_map.insert(node.uuid, i);
	}

	let uniform_buffer = device.create_buffer(&BufferDescriptor {
		label: Some("inox2d uniform buffer"),
		size: (uniform_alignment_needed * uniform_index_map.len()) as u64,
		usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("vertex buffer"),
		contents: bytemuck::cast_slice(&puppet.render_ctx.vertex_buffers.verts),
		usage: wgpu::BufferUsages::VERTEX,
	});

	let uv_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("uv buffer"),
		contents: bytemuck::cast_slice(&puppet.render_ctx.vertex_buffers.uvs),
		usage: wgpu::BufferUsages::VERTEX,
	});

	let deform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("deform buffer"),
		contents: bytemuck::cast_slice(&puppet.render_ctx.vertex_buffers.deforms),
		usage: wgpu::BufferUsages::VERTEX | BufferUsages::COPY_DST,
	});

	let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("index buffer"),
		contents: bytemuck::cast_slice(&puppet.render_ctx.vertex_buffers.indices),
		usage: wgpu::BufferUsages::INDEX,
	});

	InoxBuffers {
		uniform_buffer,
		uniform_index_map,

		vertex_buffer,
		uv_buffer,
		deform_buffer,
		index_buffer,
	}
}
