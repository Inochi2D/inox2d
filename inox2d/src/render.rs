mod vertex_buffers;

use glam::Mat4;

use crate::math::transform::Transform;
use crate::model::Model;
use crate::node::{
	components::{
		drawable::{Mask, Masks},
		Composite, Drawable, TexturedMesh,
	},
	InoxNodeUuid,
};
use crate::puppet::{Puppet, World};

use vertex_buffers::VertexBuffers;

/// Possible component combinations of a renderable node.
///
/// Future extensions go here.
enum DrawableKind<'comps> {
	TexturedMesh(TexturedMeshComponents<'comps>),
	Composite(CompositeComponents<'comps>),
}

/// Pack of components for a TexturedMesh. "Part" in Inochi2D terms.
pub struct TexturedMeshComponents<'comps> {
	pub transform: &'comps Transform,
	pub drawable: &'comps Drawable,
	pub data: &'comps TexturedMesh,
}

/// Pack of components for a Composite node.
pub struct CompositeComponents<'comps> {
	pub transform: &'comps Transform,
	pub drawable: &'comps Drawable,
	pub data: &'comps Composite,
}

impl<'comps> DrawableKind<'comps> {
	/// Tries to construct a renderable node data pack from the World of components:
	/// - `None` if node not renderable.
	/// - Panicks if component combinations invalid.
	fn new(id: InoxNodeUuid, comps: &'comps World) -> Option<Self> {
		let drawable = match comps.get::<Drawable>(id) {
			Some(drawable) => drawable,
			None => return None,
		};
		let transform = comps
			.get::<Transform>(id)
			.expect("A drawble must have associated Transform.");
		let textured_mesh = comps.get::<TexturedMesh>(id);
		let composite = comps.get::<Composite>(id);

		match (textured_mesh.is_some(), composite.is_some()) {
			(true, true) => panic!("The drawable has both TexturedMesh and Composite."),
			(false, false) => panic!("The drawable has neither TexturedMesh nor Composite."),
			(true, false) => Some(DrawableKind::TexturedMesh(TexturedMeshComponents {
				transform,
				drawable,
				data: textured_mesh.unwrap(),
			})),
			(false, true) => Some(DrawableKind::Composite(CompositeComponents {
				transform,
				drawable,
				data: composite.unwrap(),
			})),
		}
	}
}

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
	vertex_buffers: VertexBuffers,
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
		let mut drawable_uuid_zsort_vec = Vec::<(InoxNodeUuid, f32)>::new();

		for node in nodes.iter() {
			let drawable_kind = DrawableKind::new(node.uuid, comps);
			if let Some(drawable_kind) = drawable_kind {
				drawable_uuid_zsort_vec.push((node.uuid, node.zsort));

				match drawable_kind {
					DrawableKind::TexturedMesh(components) => {
						let (index_offset, vert_offset) = vertex_buffers.push(&components.data.mesh);

						comps.add(
							node.uuid,
							TexturedMeshRenderCtx {
								index_offset,
								vert_offset,
								index_len: components.data.mesh.indices.len(),
								vert_len: components.data.mesh.vertices.len(),
							},
						);
					}
					DrawableKind::Composite { .. } => {
						// exclude non-drawable children
						let mut zsorted_children_list: Vec<InoxNodeUuid> = nodes
							.get_children(node.uuid)
							.filter_map(|n| {
								if DrawableKind::new(n.uuid, comps).is_some() {
									Some(n.uuid)
								} else {
									None
								}
							})
							.collect();
						zsorted_children_list.sort_by(|a, b| {
							let zsort_a = nodes.get_node(*a).unwrap().zsort;
							let zsort_b = nodes.get_node(*b).unwrap().zsort;
							zsort_a.total_cmp(&zsort_b).reverse()
						});

						comps.add(node.uuid, CompositeRenderCtx { zsorted_children_list });
					}
				};
			}
		}

		drawable_uuid_zsort_vec.sort_by(|a, b| a.1.total_cmp(&b.1).reverse());

		Self {
			vertex_buffers,
			root_drawables_zsorted: drawable_uuid_zsort_vec.into_iter().map(|p| p.0).collect(),
		}
	}

	/// Memory layout: `[[x, y], [x, y], ...]`
	pub fn get_raw_verts(&self) -> &[f32] {
		VertexBuffers::vec_vec2_as_vec_f32(&self.vertex_buffers.verts)
	}
	/// Memory layout: `[[x, y], [x, y], ...]`
	pub fn get_raw_uvs(&self) -> &[f32] {
		VertexBuffers::vec_vec2_as_vec_f32(&self.vertex_buffers.uvs)
	}
	/// Memory layout: `[[i0, i1, i2], [i0, i1, i2], ...]`
	pub fn get_raw_indices(&self) -> &[u16] {
		self.vertex_buffers.indices.as_slice()
	}
	/// Memory layout: `[[dx, dy], [dx, dy], ...]`
	pub fn get_raw_deforms(&self) -> &[f32] {
		VertexBuffers::vec_vec2_as_vec_f32(&self.vertex_buffers.deforms)
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
pub trait InoxRenderer
where
	Self: Sized,
{
	type Error;

	/// Create a renderer for one model.
	///
	/// Ref impl: Upload textures.
	fn new(model: &Model) -> Result<Self, Self::Error>;

	/// Sets a quaternion that can translate, rotate and scale the whole puppet.
	fn set_camera(&mut self, camera: &Mat4);
	/// Returns the quaternion set by `.set_camera()`.
	fn camera(&self) -> &Mat4;

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
	renderer.draw(puppet);
}
