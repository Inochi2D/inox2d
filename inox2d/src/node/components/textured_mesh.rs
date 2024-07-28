use glam::Vec2;

use crate::texture::TextureId;

/// If has this as a component, the node should render a deformed texture
pub struct TexturedMesh {
	pub mesh: Mesh,
	pub tex_albedo: TextureId,
	pub tex_emissive: TextureId,
	pub tex_bumpmap: TextureId,
}

pub struct Mesh {
	/// Vertices in the mesh.
	pub vertices: Vec<Vec2>,
	/// Base UVs.
	pub uvs: Vec<Vec2>,
	/// Indices in the mesh.
	pub indices: Vec<u16>,
	/// Origin of the mesh.
	pub origin: Vec2,
}
