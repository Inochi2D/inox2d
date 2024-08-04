mod deform_stack;
mod vertex_buffers;

use std::mem::swap;

use crate::node::{
	components::{DeformStack, Mask, Masks, ZSort},
	drawables::{CompositeComponents, DrawableKind, TexturedMeshComponents},
	InoxNodeUuid,
};
use crate::puppet::{InoxNodeTree, Puppet, World};

pub use vertex_buffers::VertexBuffers;

/// Additional info per node for rendering a TexturedMesh:
/// - offset and length of array for mesh point coordinates
/// - offset and length of array for indices of mesh points defining the mesh
///
/// inside `puppet.render_ctx_vertex_buffers`.
pub struct TexturedMeshRenderCtx {
	pub index_offset: u16,
	pub vert_offset: u16,
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

		let mut vertex_buffers = VertexBuffers::default();

		let mut root_drawables_count: usize = 0;
		for node in nodes.iter() {
			let drawable_kind = DrawableKind::new(node.uuid, comps);
			if let Some(drawable_kind) = drawable_kind {
				root_drawables_count += 1;

				match drawable_kind {
					DrawableKind::TexturedMesh(components) => {
						let (index_offset, vert_offset) = vertex_buffers.push(&components.data.mesh);
						let (index_len, vert_len) =
							(components.data.mesh.indices.len(), components.data.mesh.vertices.len());

						comps.add(
							node.uuid,
							TexturedMeshRenderCtx {
								index_offset,
								vert_offset,
								index_len,
								vert_len,
							},
						);
						comps.add(node.uuid, DeformStack::new(vert_len));
					}
					DrawableKind::Composite { .. } => {
						// exclude non-drawable children
						let children_list: Vec<InoxNodeUuid> = nodes
							.get_children(node.uuid)
							.filter_map(|n| {
								if DrawableKind::new(n.uuid, comps).is_some() {
									Some(n.uuid)
								} else {
									None
								}
							})
							.collect();

						// composite children are excluded from root_drawables_zsorted
						root_drawables_count -= children_list.len();

						comps.add(
							node.uuid,
							CompositeRenderCtx {
								// sort later, before render
								zsorted_children_list: children_list,
							},
						);
					}
				};
			}
		}

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
			if let Some(DrawableKind::TexturedMesh(..)) = DrawableKind::new(node.uuid, comps) {
				let deform_stack = comps
					.get_mut::<DeformStack>(node.uuid)
					.expect("A TexturedMesh must have an associated DeformStack.");
				deform_stack.reset();
			}
		}
	}

	/// Update
	/// - zsort-ordered info
	/// - deform buffer content
	/// inside self, according to updated puppet.
	pub(crate) fn update(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		let mut root_drawable_uuid_zsort_vec = Vec::<(InoxNodeUuid, f32)>::new();

		// root is definitely not a drawable.
		for node in nodes.iter().skip(1) {
			if let Some(drawable_kind) = DrawableKind::new(node.uuid, comps) {
				let parent = nodes.get_parent(node.uuid);
				let node_zsort = comps.get::<ZSort>(node.uuid).unwrap().0;

				if !matches!(DrawableKind::new(parent.uuid, comps), Some(DrawableKind::Composite(_))) {
					// exclude composite children
					root_drawable_uuid_zsort_vec.push((node.uuid, node_zsort));
				}

				match drawable_kind {
					// for Composite, update zsorted children list
					DrawableKind::Composite { .. } => {
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
					// for TexturedMesh, obtain and write deforms into vertex_buffer
					DrawableKind::TexturedMesh(..) => {
						let render_ctx = comps.get::<TexturedMeshRenderCtx>(node.uuid).unwrap();
						let vert_offset = render_ctx.vert_offset as usize;
						let vert_len = render_ctx.vert_len;
						let deform_stack = comps
							.get::<DeformStack>(node.uuid)
							.expect("A TexturedMesh must have an associated DeformStack.");

						deform_stack.combine(
							nodes,
							comps,
							&mut self.vertex_buffers.deforms[vert_offset..(vert_offset + vert_len)],
						);
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

	/// Things to do before one pass of drawing a puppet.
	///
	/// Ref impl: Upload deform buffer content.
	fn on_begin_draw(&self, puppet: &Puppet);
	/// Things to do after one pass of drawing a puppet.
	fn on_end_draw(&self, puppet: &Puppet);
}

trait InoxRendererCommon {
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

impl<T: InoxRenderer> InoxRendererCommon for T {
	fn draw_drawable(&self, as_mask: bool, comps: &World, id: InoxNodeUuid) {
		let drawable_kind = DrawableKind::new(id, comps).expect("Node must be a Drawable.");
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
			let drawable_kind =
				DrawableKind::new(*uuid, comps).expect("All children in zsorted_children_list should be a Drawable.");
			match drawable_kind {
				DrawableKind::TexturedMesh(components) => {
					self.draw_textured_mesh_content(as_mask, &components, comps.get(*uuid).unwrap(), *uuid)
				}
				DrawableKind::Composite { .. } => panic!("Composite inside Composite not allowed."),
			}
		}

		self.finish_composite_content(as_mask, components, render_ctx, id);
	}

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

/// Dispatches draw calls for all nodes of `puppet`
/// - with provided renderer implementation,
/// - in Inochi2D standard defined order.
///
/// This does not guarantee the display of a puppet on screen due to these possible reasons:
/// - Only provided `InoxRenderer` method implementations are called.
/// For example, maybe the caller still need to transfer content from a texture buffer to the screen surface buffer.
/// - The provided `InoxRender` implementation is wrong.
/// - `puppet` here does not belong to the `model` this `renderer` is initialized with. This will likely result in panics for non-existent node uuids.
pub fn draw<T: InoxRenderer>(renderer: &T, puppet: &Puppet) {
	renderer.on_begin_draw(puppet);
	renderer.draw(puppet);
	renderer.on_end_draw(puppet);
}
