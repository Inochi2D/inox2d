use glam::Vec2;

/// Different kinds of deform.
// TODO: Meshgroup.
pub(crate) enum Deform {
	/// Specifying a displacement for every vertex.
	Direct(Vec<Vec2>),
	/// Specifying a deformation based on a parent mesh group.
	#[allow(dead_code)]
	MeshGroup {
		/// UUID of the mesh group node.
		mesh_group: crate::node::InoxNodeUuid,
		/// Barycentric weights for each vertex of the child.
		/// Each element is (triangle_index, weights).
		vertex_weights: Vec<(usize, [f32; 3])>,
	},
}

/// Calculate barycentric weights of a point in a triangle.
#[allow(dead_code)]
pub(crate) fn barycentric_weights(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> [f32; 3] {
	let v0 = b - a;
	let v1 = c - a;
	let v2 = p - a;
	let d00 = v0.dot(v0);
	let d01 = v0.dot(v1);
	let d11 = v1.dot(v1);
	let d20 = v2.dot(v0);
	let d21 = v2.dot(v1);
	let denom = d00 * d11 - d01 * d01;

	// fallback for degenerate triangles
	if denom.abs() < 1e-6 {
		return [1.0, 0.0, 0.0];
	}

	let v = (d11 * d20 - d01 * d21) / denom;
	let w = (d00 * d21 - d01 * d20) / denom;
	let u = 1.0 - v - w;
	[u, v, w]
}

/// Element-wise add direct deforms up and write result.
pub(crate) fn linear_combine<'deforms>(direct_deforms: impl Iterator<Item = &'deforms Vec<Vec2>>, result: &mut [Vec2]) {
	result.iter_mut().for_each(|deform| *deform = Vec2::ZERO);

	for direct_deform in direct_deforms {
		if direct_deform.len() != result.len() {
			panic!("Trying to combine direct deformations with wrong dimensions.");
		}

		result
			.iter_mut()
			.zip(direct_deform.iter())
			.for_each(|(sum, addition)| *sum += *addition);
	}
}
