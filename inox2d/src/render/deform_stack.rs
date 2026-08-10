use std::collections::HashMap;
use std::mem::swap;
use std::ops::Range;

use glam::Vec2;

use crate::math::deform::{foreign_mesh_combine, linear_combine, Deform};
use crate::node::components::{DeformSource, DeformStack, Mesh};
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
	) {
		let render_ctx = node_comps.get::<TexturedMeshRenderCtx>(uuid).unwrap();
		let vert_offset = render_ctx.vert_offset as usize;
		let vert_len = render_ctx.vert_len;
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
						// TODO: This doesn't work.
						// MeshGroups don't get a TexturedMeshRenderCtx, and
						// they probably don't have allocated space in the
						// deform array - because they're not a renderable, and
						// we're not going to send their deforms to the GPU.
						// Instead, each MeshGroup needs to hold its own deform
						// scratch buffer to combine to?
						// Furthermore, we need somewhere to call .combine
						// BEFORE render_ctx.apply() (so the meshes can
						// reference the new deform data) and make sure we are
						// combining node parents before children.
						// Since we don't want to have to build a separate DAG
						// of DeformStack dependencies, let's just say that
						// Source deforms may ONLY reference parent nodes, and
						// that parents always get applied before children.
						// That works well with the current node order.
						// Ideally all the deform logic would be refactored out
						// into the renderer (since deforms can't do anything
						// on the CPU) and then the renderer can GPU-skin
						// everything
						let Some(foreign_render_ctx) = node_comps.get::<TexturedMeshRenderCtx>(*source_node) else {
							continue;
						};

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
