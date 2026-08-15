use std::collections::HashMap;
use std::mem::swap;
use std::ops::Range;

use glam::Vec2;

use crate::math::deform::{foreign_mesh_combine, linear_combine, Deform};
use crate::node::components::{DeformSource, DeformStack, Mesh, MeshGroupDeform};
use crate::puppet::{InoxNodeTree, World};
use crate::render::{InoxNodeUuid, TexturedMeshRenderCtx};

fn split_ranges<'a, T>(
	array: &'a mut [T],
	mut r1: Range<usize>,
	mut r2: Range<usize>,
) -> Option<(&'a mut [T], &'a mut [T])> {
	let swapped = r2.start < r1.start;
	if swapped {
		swap(&mut r1, &mut r2);
	}

	//At this point R2 is guaranteed to be after R1
	if r2.start < r1.end {
		return None;
	}

	let (array_1, array_2) = array.split_at_mut(r1.end);
	let (_array_garbo, array_1) = array_1.split_at_mut(r1.start);
	let (_array_garbo, array_2) = array_2.split_at_mut(r2.start - r1.end);
	let (array_2, _array_garbo) = array_2.split_at_mut(r2.end - r2.start);

	if swapped {
		Some((array_2, array_1))
	} else {
		Some((array_1, array_2))
	}
}

impl DeformStack {
	pub(crate) fn new(deform_len: usize) -> Self {
		Self {
			deform_len,
			stack: HashMap::new(),
		}
	}

	/// Reset the stack. Ready to receive deformations for one frame.
	pub(crate) fn reset(&mut self) {
		for enabled_deform in self.stack.values_mut() {
			enabled_deform.0 = false;
		}
	}

	/// Combine the deformations received so far according to some rules, and write to the result
	pub(crate) fn combine(
		&self,
		uuid: InoxNodeUuid,
		_nodes: &InoxNodeTree,
		node_comps: &World,
		result: &mut [Vec2],
		vert_offset: usize,
		vert_len: usize,
	) {
		if vert_len != self.deform_len {
			panic!(
				"Required output deform dimensions {} different from what DeformStack is initialized with ({}).",
				vert_len, self.deform_len
			);
		}

		result[vert_offset..(vert_offset + vert_len)]
			.iter_mut()
			.for_each(|deform| *deform = Vec2::ZERO);

		for (src, (enabled, deform)) in self.stack.iter() {
			if *enabled {
				match *deform {
					Deform::Direct(ref direct_deform) => {
						linear_combine(direct_deform, &mut result[vert_offset..(vert_offset + vert_len)]);
					}
					Deform::Source => {
						let DeformSource::MeshGroup(source_node) = src else {
							panic!("Source deform application through params is not supported (or meaningful)");
						};

						let Some(result_mesh) = node_comps.get::<Mesh>(uuid) else {
							continue;
						};
						let Some(foreign_mesh) = node_comps.get::<Mesh>(*source_node) else {
							continue;
						};

						if *source_node == uuid {
							eprintln!("Self-referential deformation is not permitted on node {:?}", uuid);
							continue;
						}

						let Some((my_deform, foreign_deform)) =
							(if let Some(foreign_render_ctx) = node_comps.get::<TexturedMeshRenderCtx>(*source_node) {
								let foreign_offset = foreign_render_ctx.vert_offset as usize;
								let foreign_len = foreign_render_ctx.vert_len;

								let Some((my_deform, foreign_deform)) = split_ranges(
									result,
									vert_offset..(vert_offset + vert_len),
									foreign_offset..(foreign_offset + foreign_len),
								) else {
									eprintln!(
										"They're disjoint?! {:?}, {:?}",
										vert_offset..(vert_offset + vert_len),
										foreign_offset..(foreign_offset + foreign_len)
									);
									continue;
								};

								Some((my_deform, &*foreign_deform))
							} else if let Some(mesh_deform_buffer) = node_comps.get::<MeshGroupDeform>(*source_node) {
								Some((
									&mut result[vert_offset..(vert_offset + vert_len)],
									&mesh_deform_buffer.deform[..],
								))
							} else {
								None
							})
						else {
							continue;
						};

						// If the foreign deform is empty, don't apply the deform.
						if foreign_deform.len() == 0 {
							continue;
						}

						eprintln!("SIZES: {}, {}", my_deform.len(), foreign_deform.len());

						foreign_mesh_combine(foreign_mesh, foreign_deform, result_mesh, my_deform);
					}
				}
			}
		}
	}

	/// Submit a deform from a source for a node.
	pub(crate) fn push(&mut self, src: DeformSource, mut deform: Deform) {
		match deform {
			Deform::Direct(ref direct_deform) => {
				if direct_deform.len() != self.deform_len {
					panic!("A direct deform with non-matching dimensions is submitted to a node.");
				}

				self.stack
					.entry(src)
					.and_modify(|enabled_deform| {
						if enabled_deform.0 {
							panic!("A same source submitted deform twice for a same node within one frame.")
						}
						enabled_deform.0 = true;

						swap(&mut enabled_deform.1, &mut deform);
					})
					.or_insert((true, deform));
			}
			Deform::Source => {
				let DeformSource::MeshGroup(_source_node) = src else {
					panic!("Source deform application through params is not supported (or meaningful)");
				};

				self.stack
					.entry(src)
					.and_modify(|(enabled, old_deform)| {
						if *enabled {
							panic!("A same source submitted deform twice for a same node within one frame.")
						}
						*enabled = true;

						swap(old_deform, &mut deform);
					})
					.or_insert((true, deform));
			}
		}
	}
}
