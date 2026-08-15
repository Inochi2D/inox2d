use crate::{math::triangle::MeshBitMask, node::components::Mesh};
use glam::{Mat2, Vec2};
use std::collections::HashMap;

/// Different kinds of deform.
// TODO: Meshgroup.
pub(crate) enum Deform {
	/// Specifying a displacement for every vertex.
	Direct(Vec<Vec2>),

	/// Apply the source's deformation to each vertex.
	///
	/// This currently only implements "dynamic deformation" as mandated in 0.9.
	/// In 0.8, there is an option to turn that off; which precalculates the
	/// parent deform and saves some CPU time.
	///
	/// TODO: I'm not 100% sure if that explanation is accurate.
	Source,
}

/// Element-wise add a single direct deform up and write result.
pub(crate) fn linear_combine(direct_deform: &[Vec2], result: &mut [Vec2]) {
	if direct_deform.len() != result.len() {
		panic!("Trying to combine direct deformations with wrong dimensions.");
	}

	result
		.iter_mut()
		.zip(direct_deform.iter())
		.for_each(|(sum, addition)| *sum += *addition);
}

/// Element-wise apply a foreign mesh's deforms to the result mesh and write
/// result.
///
/// To help visualize this, imagine if the result mesh's vertices were somehow
/// part of the texture of the foreign mesh, and we applied the deform that way.
/// Each result vert is assigned to a triangle of the foreign mesh, and then
/// that triangle's deforms are applied to the result vert based on the
/// barycentric distance to each foreign triangle vert.
///
/// Also, the result mesh is treated as if the result deform were already
/// applied, meaning this transform is non-linear.
pub(crate) fn foreign_mesh_combine(
	foreign_mesh: &Mesh,
	foreign_deform: &[Vec2],
	result_mesh: &Mesh,
	result: &mut [Vec2],
) {
	if result_mesh.vertices.len() != result.len() {
		panic!("Trying to combine a foreign mesh deformation with wrong dimensions.");
	}

	let testing_mask = MeshBitMask::new(foreign_mesh);
	let mut triangle_bins = HashMap::new();
	for (index, (result_vert, result_deform)) in result_mesh.vertices.iter().zip(result.iter()).enumerate() {
		let result_vert_applied = *result_vert + *result_deform;

		if let Some(triangle_start_index) = testing_mask.test(result_vert_applied) {
			let (indexes, verts) = triangle_bins.entry(triangle_start_index).or_insert((vec![], vec![]));

			indexes.push(index);
			verts.push(result_vert_applied);
		}
	}

	for (triangle_start_index, (vert_indexes, verts)) in triangle_bins.iter_mut() {
		let triangle = foreign_mesh.get_triangle(*triangle_start_index);
		let triangle_deforms = foreign_mesh.get_triangle_deforms(*triangle_start_index, foreign_deform);
		let decompose_matrix = vector_decompose_matrix(triangle[1] - triangle[0], triangle[2] - triangle[0]);

		//TODO: I'm pretty sure we're supposed to be giving points in batches,
		//but I'm too lazy to write a binning scheme for triangles.
		let deforms = deform_by_parent_triangle(&decompose_matrix, triangle[0], &triangle_deforms, verts.iter());

		for (vert_index, deform) in vert_indexes.into_iter().zip(deforms) {
			result[*vert_index] += deform;
		}
	}
}

/// Input: two basis vectors `b0` and `b1`.
///
/// If the returned matrix is to be applied on a Vec2 V and X is obtained,
/// then `X.x * b0 + X.y * b1 = V`.
///
/// Panics if either basis is zero or they are not independent of each other.
pub fn vector_decompose_matrix(b0: Vec2, b1: Vec2) -> Mat2 {
	// B X = V where:
	// B: [ b0.x b1.x
	//      b0.y b1.y ]
	// X: [ x
	//      y ]
	// V: [ v.x
	//      v.y ]
	// thus X = B^-1 V
	let mat = Mat2::from_cols(b0, b1).inverse();
	debug_assert_ne!(mat.determinant(), 0.0, "Provided two basis do not span the 2D plane.");
	mat
}

/// Provide a parent triangle and its deforms by 3 points,
/// calculate how far should the provided points be moved by the triangle's deform.
///
/// For optimization, the "decompose_matrix" of parent should be provided, see `vector_decompose_matrix()`.
/// It is assumed that `parent[0]` is taken as the origin,
/// `parent[1] - parent[0]` is the first basis vector, and `parent[2] - parent[0]` the second.
#[inline]
pub fn deform_by_parent_triangle<'a>(
	decompose_matrix: &'a Mat2,
	parent_p0: Vec2,
	parent_deforms: &'a [Vec2; 3],
	points: impl Iterator<Item = &'a Vec2> + 'a,
) -> impl Iterator<Item = Vec2> + 'a {
	let basis_0_deform = parent_deforms[1] - parent_deforms[0];
	let basis_1_deform = parent_deforms[2] - parent_deforms[0];

	points.map(move |p| {
		let decomposed_coeffs = *decompose_matrix * (*p - parent_p0);
		// deform by parent[0] + deform by basis change
		parent_deforms[0] + decomposed_coeffs.x * basis_0_deform + decomposed_coeffs.y * basis_1_deform
	})
}
