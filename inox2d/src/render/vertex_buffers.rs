use glam::{vec2, Vec2};

use crate::node::components::Mesh;

pub struct VertexBuffers {
	pub verts: Vec<Vec2>,
	pub uvs: Vec<Vec2>,
	pub indices: Vec<u16>,
	pub deforms: Vec<Vec2>,
}

impl Default for VertexBuffers {
	fn default() -> Self {
		// init with a quad covering the whole viewport

		#[rustfmt::skip]
		let verts = vec![
            vec2(-1.0, -1.0),
            vec2(-1.0, 1.0),
            vec2(1.0, -1.0),
            vec2(1.0, 1.0)
        ];

		#[rustfmt::skip]
		let uvs = vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 0.0),
            vec2(1.0, 1.0)
        ];

		#[rustfmt::skip]
		let indices = vec![
            0, 1, 2,
            2, 1, 3
        ];

		let deforms = vec![Vec2::ZERO; 4];

		Self {
			verts,
			uvs,
			indices,
			deforms,
		}
	}
}

impl VertexBuffers {
	/// Adds the mesh's vertices and UVs to the buffers and returns its index and vertex offset.
	pub fn push(&mut self, mesh: &Mesh) -> (u16, u16) {
		let index_offset = self.indices.len() as u16;
		let vert_offset = self.verts.len() as u16;

		self.verts.extend_from_slice(&mesh.vertices);
		self.uvs.extend_from_slice(&mesh.uvs);
		self.indices
			.extend(mesh.indices.iter().map(|index| index + vert_offset));
		self.deforms
			.resize(self.deforms.len() + mesh.vertices.len(), Vec2::ZERO);

		(index_offset, vert_offset)
	}
}
