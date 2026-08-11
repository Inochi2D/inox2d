mod deform_stack;
mod vertex_buffers;

use std::collections::{HashMap, HashSet};
use std::mem::swap;

use crate::node::{
	components::{DeformStack, Mask, Masks, ZSort},
	drawables::{CompositeComponents, DrawableKind, TexturedMeshComponents},
	InoxNodeUuid,
};
use crate::params::BindingValues;
use crate::puppet::{InoxNodeTree, Puppet, World};

pub use vertex_buffers::VertexBuffers;

/// Additional info per node for rendering a TexturedMesh:
/// - offset and length of array for mesh point coordinates
/// - offset and length of array for indices of mesh points defining the mesh
///
/// inside `puppet.render_ctx_vertex_buffers`.
pub struct TexturedMeshRenderCtx {
	pub index_offset: u32,
	pub vert_offset: u32,
	pub index_len: usize,
	pub vert_len: usize,
}

/// Additional info per node for rendering a Composite.
pub struct CompositeRenderCtx {
	pub zsorted_children_list: Vec<InoxNodeUuid>,
}

/// Additional struct attached to a puppet for rendering.
pub struct RenderCtx {
	/// General compact data buffers for interfacing with the GPU.
	pub vertex_buffers: VertexBuffers,
	/// All nodes that need respective draw method calls:
	/// - including standalone parts and composite parents,
	/// - excluding (TODO: plain mesh masks) and composite children.
	root_drawables_zsorted: Vec<InoxNodeUuid>,
}

impl RenderCtx {
	/// MODIFIES puppet. In addition to initializing self, installs render contexts in the World of components
	pub(super) fn new(puppet: &mut Puppet) -> Self {
		let nodes = &puppet.nodes;
		let comps = &mut puppet.node_comps;

		let mut nodes_to_deform = HashSet::new();
		for param in &puppet.params {
			param.1.bindings.iter().for_each(|b| {
				if matches!(b.values, BindingValues::Deform(_)) {
					nodes_to_deform.insert(b.node);
				}
			});
		}
		// TODO: Further fill the set when Meshgroup is implemented.

		let mut vertex_buffers = VertexBuffers::default();

		let mut root_drawables_count: usize = 0;

		// Composite uuid => (descendant list, drawable count)
		let mut composite_descendent_lists: HashMap<InoxNodeUuid, (Vec<InoxNodeUuid>, usize)> = HashMap::new();
		// Vec<(Node uuid, maybe highest Composite ancestor)>
		// This stack skips on nodes that don't have a Composite ancestor
		let mut ancestor_stack: Vec<(InoxNodeUuid, Option<InoxNodeUuid>)> = Vec::new();

		for node in nodes.pre_order_iter() {
			// Pop until top of stack is parent of current node
			while let Some(top) = ancestor_stack.last() {
				if nodes.get_parent(node.uuid).uuid == top.0 {
					break;
				}
				ancestor_stack.pop();
			}

			let top_composite = ancestor_stack.last().and_then(|curr_top| curr_top.1);

			if let Some(top_composite) = top_composite {
				// Current node is a descendant of current highest Composite (could be anything, a Node, Part, Meshgroup, Composite etc.)
				composite_descendent_lists
					.entry(top_composite)
					.or_default()
					.0
					.push(node.uuid);
				ancestor_stack.push((node.uuid, Some(top_composite)));
			}

			let drawable_kind = DrawableKind::new(node.uuid, comps, true);
			if let Some(drawable_kind) = drawable_kind {
				root_drawables_count += 1;

				if let Some(target_id) = top_composite {
					composite_descendent_lists.entry(target_id).or_default().1 += 1;
				}

				match drawable_kind {
					DrawableKind::TexturedMesh(components) => {
						let (index_offset, vert_offset) = vertex_buffers.push(components.mesh);
						let (index_len, vert_len) = (components.mesh.indices.len(), components.mesh.vertices.len());

						comps.add(
							node.uuid,
							TexturedMeshRenderCtx {
								index_offset,
								vert_offset,
								index_len,
								vert_len,
							},
						);

						// TexturedMesh not deformed by any source does not need a DeformStack
						if nodes_to_deform.contains(&node.uuid) {
							comps.add(node.uuid, DeformStack::new(vert_len));
						}
					}
					DrawableKind::Composite { .. } => {
						if top_composite.is_none() {
							// Empty stack -> current node is the highest Composite on its branch
							// (otherwise this Composite would be pushed twice) 
							ancestor_stack.push((node.uuid, Some(node.uuid)));
						}
					}
				};
			}
		}

		composite_descendent_lists
			.iter()
			.for_each(|(composite_id, (descendents, drawable_count))| {
				root_drawables_count -= drawable_count;
				comps.add(
					*composite_id,
					CompositeRenderCtx {
						zsorted_children_list: descendents.clone(),
					},
				);
			});

		let mut root_drawables_zsorted = Vec::new();
		// similarly, populate later, before render
		root_drawables_zsorted.resize(root_drawables_count, InoxNodeUuid(0));

		Self {
			vertex_buffers,
			root_drawables_zsorted,
		}
	}

	/// Reset all `DeformStack`.
	pub(crate) fn reset(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		for node in nodes.iter() {
			if let Some(deform_stack) = comps.get_mut::<DeformStack>(node.uuid) {
				deform_stack.reset();
			}
		}
	}

	/// Update zsort-ordered info and deform buffer content inside self, according to updated puppet.
	pub(crate) fn update(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		let mut root_drawable_uuid_zsort_vec = Vec::<(InoxNodeUuid, f32)>::new();

		let mut ancestor_stack: Vec<(InoxNodeUuid, Option<InoxNodeUuid>)> = Vec::new();

		// root is definitely not a drawable.
		for node in nodes.pre_order_iter().skip(1) {
			while let Some(top) = ancestor_stack.last() {
				if nodes.get_parent(node.uuid).uuid == top.0 {
					break;
				}
				ancestor_stack.pop();
			}

			let top_composite = ancestor_stack.last().and_then(|top| top.1);
			if let Some(top_composite) = top_composite {
				ancestor_stack.push((node.uuid, Some(top_composite)));
			}
			if let Some(drawable_kind) = DrawableKind::new(node.uuid, comps, false) {
				let node_zsort = comps.get::<ZSort>(node.uuid).unwrap().0;

				if top_composite.is_none() {
					root_drawable_uuid_zsort_vec.push((node.uuid, node_zsort));
				}

				match drawable_kind {
					// for Composite, update zsorted children list
					DrawableKind::Composite { .. } => {
						// Nested composites behave like a normal part zsort-wise
						// i.e. not owning or managing a zsorted children list
						if top_composite.is_none() {
							ancestor_stack.push((node.uuid, Some(node.uuid)));

							// `swap()` usage is a trick that both:
							// - returns mut borrowed comps early
							// - does not involve any heap allocations
							let mut zsorted_children_list = Vec::new();
							swap(
								&mut zsorted_children_list,
								&mut comps
									.get_mut::<CompositeRenderCtx>(node.uuid)
									.unwrap()
									.zsorted_children_list,
							);

							zsorted_children_list.sort_by(|a, b| {
								let zsort_a = comps.get::<ZSort>(*a).unwrap();
								let zsort_b = comps.get::<ZSort>(*b).unwrap();
								zsort_a.total_cmp(zsort_b).reverse()
							});

							swap(
								&mut zsorted_children_list,
								&mut comps
									.get_mut::<CompositeRenderCtx>(node.uuid)
									.unwrap()
									.zsorted_children_list,
							);
						}
					}
					// for TexturedMesh, obtain and write deforms into vertex_buffer
					DrawableKind::TexturedMesh(..) => {
						// A TexturedMesh not having an associated DeformStack means it will not be deformed at all, skip.
						if let Some(deform_stack) = comps.get::<DeformStack>(node.uuid) {
							let render_ctx = comps.get::<TexturedMeshRenderCtx>(node.uuid).unwrap();
							let vert_offset = render_ctx.vert_offset as usize;
							let vert_len = render_ctx.vert_len;
							deform_stack.combine(
								nodes,
								comps,
								&mut self.vertex_buffers.deforms[vert_offset..(vert_offset + vert_len)],
							);
						}
					}
				}
			}
		}

		root_drawable_uuid_zsort_vec.sort_by(|a, b| a.1.total_cmp(&b.1).reverse());
		self.root_drawables_zsorted
			.iter_mut()
			.zip(root_drawable_uuid_zsort_vec.iter())
			.for_each(|(old, new)| *old = new.0);
	}
}

/// Same as the reference Inochi2D implementation, Inox2D also aims for a "bring your own rendering backend" design.
/// A custom backend shall implement this trait.
///
/// It is perfectly fine that the trait implementation does not contain everything needed to display a puppet as:
/// - The renderer may not be directly rendering to the screen for flexibility.
/// - The renderer may want platform-specific optimizations, e.g. batching, and the provided implementation is merely for collecting puppet info.
/// - The renderer may be a debug/just-for-fun renderer intercepting draw calls for other purposes.
///
/// Either way, the point is Inox2D will implement a `draw()` method for any `impl InoxRenderer`, dispatching calls based on puppet structure according to Inochi2D standard.
pub trait InoxRenderer {
	/// Begin masking.
	///
	/// Ref impl: Clear and start writing to the stencil buffer, lock the color buffer.
	fn on_begin_masks(&self, masks: &Masks);
	/// Get prepared for rendering a singular Mask.
	fn on_begin_mask(&self, mask: &Mask);
	/// Get prepared for rendering masked content.
	///
	/// Ref impl: Read only from the stencil buffer, unlock the color buffer.
	fn on_begin_masked_content(&self);
	/// End masking.
	///
	/// Ref impl: Disable the stencil buffer.
	fn on_end_mask(&self);

	/// Draw TexturedMesh content.
	// TODO: TexturedMesh without any texture (usually for mesh masks)?
	fn draw_textured_mesh_content(
		&self,
		as_mask: bool,
		components: &TexturedMeshComponents,
		render_ctx: &TexturedMeshRenderCtx,
		id: InoxNodeUuid,
	);

	/// Begin compositing. Get prepared for rendering children of a Composite.
	///
	/// Ref impl: Prepare composite buffers.
	fn begin_composite_content(
		&self,
		as_mask: bool,
		components: &CompositeComponents,
		render_ctx: &CompositeRenderCtx,
		id: InoxNodeUuid,
	);
	/// End compositing.
	///
	/// Ref impl: Transfer content from composite buffers to normal buffers.
	fn finish_composite_content(
		&self,
		as_mask: bool,
		components: &CompositeComponents,
		render_ctx: &CompositeRenderCtx,
		id: InoxNodeUuid,
	);
}

pub trait InoxRendererExt {
	/// Draw a Drawable, which is potentially masked.
	fn draw_drawable(&self, as_mask: bool, comps: &World, id: InoxNodeUuid);

	/// Draw one composite. `components` must be referencing `comps`.
	fn draw_composite(&self, as_mask: bool, comps: &World, components: &CompositeComponents, id: InoxNodeUuid);

	/// Iterate over top-level drawables (excluding masks) in zsort order,
	/// and make draw calls correspondingly.
	///
	/// This effectively draws the complete puppet.
	fn draw(&self, puppet: &Puppet);
}

impl<T: InoxRenderer> InoxRendererExt for T {
	fn draw_drawable(&self, as_mask: bool, comps: &World, id: InoxNodeUuid) {
		let drawable_kind = DrawableKind::new(id, comps, false).expect("Node must be a Drawable.");
		let masks = match drawable_kind {
			DrawableKind::TexturedMesh(ref components) => &components.drawable.masks,
			DrawableKind::Composite(ref components) => &components.drawable.masks,
		};

		let mut has_masks = false;
		if let Some(ref masks) = masks {
			has_masks = true;
			self.on_begin_masks(masks);
			for mask in &masks.masks {
				self.on_begin_mask(mask);

				self.draw_drawable(true, comps, mask.source);
			}
			self.on_begin_masked_content();
		}

		match drawable_kind {
			DrawableKind::TexturedMesh(ref components) => {
				self.draw_textured_mesh_content(as_mask, components, comps.get(id).unwrap(), id)
			}
			DrawableKind::Composite(ref components) => self.draw_composite(as_mask, comps, components, id),
		}

		if has_masks {
			self.on_end_mask();
		}
	}

	fn draw_composite(&self, as_mask: bool, comps: &World, components: &CompositeComponents, id: InoxNodeUuid) {
		let render_ctx = comps.get::<CompositeRenderCtx>(id).unwrap();
		if render_ctx.zsorted_children_list.is_empty() {
			// Optimization: Nothing to be drawn, skip context switching
			return;
		}

		self.begin_composite_content(as_mask, components, render_ctx, id);

		for uuid in &render_ctx.zsorted_children_list {
			let drawable_kind = DrawableKind::new(*uuid, comps, false)
				.expect("All children in zsorted_children_list should be a Drawable.");
			match drawable_kind {
				DrawableKind::TexturedMesh(components) => {
					self.draw_textured_mesh_content(as_mask, &components, comps.get(*uuid).unwrap(), *uuid)
				}
				DrawableKind::Composite { .. } => continue, // Allow composite inside composite
			}
		}

		self.finish_composite_content(as_mask, components, render_ctx, id);
	}

	/// Dispatches draw calls for all nodes of `puppet`
	/// - with provided renderer implementation,
	/// - in Inochi2D standard defined order.
	///
	/// This does not guarantee the display of a puppet on screen due to these possible reasons:
	/// - Only provided `InoxRenderer` method implementations are called.
	///
	/// For example, maybe the caller still need to transfer content from a texture buffer to the screen surface buffer.
	/// - The provided `InoxRender` implementation is wrong.
	/// - `puppet` here does not belong to the `model` this `renderer` is initialized with. This will likely result in panics for non-existent node uuids.
	fn draw(&self, puppet: &Puppet) {
		for uuid in &puppet
			.render_ctx
			.as_ref()
			.expect("RenderCtx of puppet must be initialized before calling draw().")
			.root_drawables_zsorted
		{
			self.draw_drawable(false, &puppet.node_comps, *uuid);
		}
	}
}
