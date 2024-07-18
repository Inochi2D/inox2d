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
	TexturedMesh {
		transform: &'comps Transform,
		drawable: &'comps Drawable,
		data: &'comps TexturedMesh,
	},
	Composite {
		transform: &'comps Transform,
		drawable: &'comps Drawable,
		data: &'comps Composite,
	},
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
			(true, false) => Some(DrawableKind::TexturedMesh {
				transform,
				drawable,
				data: textured_mesh.unwrap(),
			}),
			(false, true) => Some(DrawableKind::Composite {
				transform,
				drawable,
				data: composite.unwrap(),
			}),
		}
	}
}

pub struct TexturedMeshRenderCtx {
	pub index_offset: u16,
	pub vert_offset: u16,
	pub index_len: usize,
	pub vert_len: usize,
}

pub struct CompositeRenderCtx {
	pub zsorted_children_list: Vec<InoxNodeUuid>,
}

pub struct RenderCtx {
	vertex_buffers: VertexBuffers,
	/// All nodes that need respective draw method calls:
	/// - including standalone parts and composite parents,
	/// - excluding plain mesh masks and composite children.
	root_drawables_zsorted: Vec<InoxNodeUuid>,
}

impl RenderCtx {
	/// MODIFIES puppet. In addition to initializing self, installs render contexts in the World of components
	fn new(puppet: &mut Puppet) -> Self {
		let nodes = &puppet.nodes;
		let comps = &mut puppet.node_comps;

		let mut vertex_buffers = VertexBuffers::default();
		let mut drawable_uuid_zsort_vec = Vec::<(InoxNodeUuid, f32)>::new();

		for node in nodes.iter() {
			let drawable_kind = DrawableKind::new(node.uuid, comps);
			if let Some(drawable_kind) = drawable_kind {
				drawable_uuid_zsort_vec.push((node.uuid, node.zsort));

				match drawable_kind {
					DrawableKind::TexturedMesh { data, .. } => {
						let (index_offset, vert_offset) = vertex_buffers.push(&data.mesh);

						comps.add(
							node.uuid,
							TexturedMeshRenderCtx {
								index_offset,
								vert_offset,
								index_len: data.mesh.indices.len(),
								vert_len: data.mesh.vertices.len(),
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

	fn get_raw_verts(&self) -> &[f32] {
		VertexBuffers::vec_vec2_as_vec_f32(&self.vertex_buffers.verts)
	}
	fn get_raw_uvs(&self) -> &[f32] {
		VertexBuffers::vec_vec2_as_vec_f32(&self.vertex_buffers.uvs)
	}
	fn get_raw_indices(&self) -> &[u16] {
		self.vertex_buffers.indices.as_slice()
	}
	fn get_raw_deforms(&self) -> &[f32] {
		VertexBuffers::vec_vec2_as_vec_f32(&self.vertex_buffers.deforms)
	}
}

impl Puppet {
	pub fn init_render_ctx(&mut self) {
		if self.render_ctx.is_some() {
			panic!("RenderCtx already initialized.");
		}

		let render_ctx = RenderCtx::new(self);
		self.render_ctx = Some(render_ctx);
	}
}

pub trait InoxRenderer
where
	Self: Sized,
{
	type Error;

	/// For any model-specific setup, e.g. creating buffers with specific sizes.
	///
	/// After this step, the provided model should be renderable.
	fn prepare(&mut self, model: &Model) -> Result<(), Self::Error>;

	/// Resize the renderer's viewport.
	fn resize(&mut self, w: u32, h: u32);

	/// Clear the canvas.
	fn clear(&self);

	/// Initiate one render pass.
	fn on_begin_scene(&self);
	/// Finish one render pass.
	fn on_end_scene(&self);
	/// Actually make results visible, e.g. on a screen/texture.
	fn draw_scene(&self);

	/// Begin masking.
	///
	/// Clear and start writing to the stencil buffer, lock the color buffer.
	fn on_begin_masks(&self, masks: &Masks);
	/// Get prepared for rendering a singular Mask.
	fn on_begin_mask(&self, mask: &Mask);
	/// Get prepared for rendering masked content.
	///
	/// Read only from the stencil buffer, unlock the color buffer.
	fn on_begin_masked_content(&self);
	/// End masking.
	///
	/// Disable the stencil buffer.
	fn on_end_mask(&self);

	/// Draw TexturedMesh content.
	///
	/// TODO: TexturedMesh without any texture (usually for mesh masks) not implemented
	fn draw_textured_mesh_content(
		&self,
		as_mask: bool,
		camera: &Mat4,
		trans: &Transform,
		drawable: &Drawable,
		textured_mesh: &TexturedMesh,
		render_ctx: &TexturedMeshRenderCtx,
	);

	/// When something needs to happen before drawing to the composite buffers.
	fn begin_composite_content(
		&self,
		as_mask: bool,
		drawable: &Drawable,
		composite: &Composite,
		render_ctx: &CompositeRenderCtx,
	);
	/// Transfer content from composite buffers to normal buffers.
	fn finish_composite_content(
		&self,
		as_mask: bool,
		drawable: &Drawable,
		composite: &Composite,
		render_ctx: &CompositeRenderCtx,
	);
}

trait InoxRendererCommon {
	/// Draw a Drawable, which is potentially masked.
	fn draw_drawable<'comps>(
		&self,
		as_mask: bool,
		camera: &Mat4,
		drawable_kind: &'comps DrawableKind,
		comps: &'comps World,
		id: InoxNodeUuid,
	);

	/// Draw one composite.
	fn draw_composite<'comps>(
		&self,
		as_mask: bool,
		camera: &Mat4,
		trans: &'comps Transform,
		drawable: &'comps Drawable,
		composite: &'comps Composite,
		render_ctx: &'comps CompositeRenderCtx,
		comps: &'comps World,
	);

	/// Iterate over top-level drawables (excluding masks) in zsort order,
	/// and make draw calls correspondingly.
	///
	/// This effectively draws the complete puppet.
	fn draw(&self, camera: &Mat4, puppet: &Puppet);
}

impl<T: InoxRenderer> InoxRendererCommon for T {
	fn draw_drawable<'comps>(
		&self,
		as_mask: bool,
		camera: &Mat4,
		drawable_kind: &'comps DrawableKind,
		comps: &'comps World,
		id: InoxNodeUuid,
	) {
		let masks = match drawable_kind {
			DrawableKind::TexturedMesh { drawable, .. } => &drawable.masks,
			DrawableKind::Composite { drawable, .. } => &drawable.masks,
		};

		let mut has_masks = false;
		if let Some(ref masks) = masks {
			has_masks = true;
			self.on_begin_masks(masks);
			for mask in &masks.masks {
				self.on_begin_mask(mask);

				let mask_id = mask.source;
				let mask_drawable_kind = DrawableKind::new(mask_id, comps).expect("A Mask source must be a Drawable.");

				self.draw_drawable(true, camera, &mask_drawable_kind, comps, mask_id);
			}
			self.on_begin_masked_content();
		}

		match drawable_kind {
			DrawableKind::TexturedMesh {
				transform,
				drawable,
				data,
			} => self.draw_textured_mesh_content(as_mask, camera, transform, drawable, data, comps.get(id).unwrap()),
			DrawableKind::Composite {
				transform,
				drawable,
				data,
			} => self.draw_composite(
				as_mask,
				camera,
				transform,
				drawable,
				data,
				comps.get(id).unwrap(),
				comps,
			),
		}

		if has_masks {
			self.on_end_mask();
		}
	}

	fn draw_composite<'comps>(
		&self,
		as_mask: bool,
		camera: &Mat4,
		_trans: &'comps Transform,
		drawable: &'comps Drawable,
		composite: &'comps Composite,
		render_ctx: &'comps CompositeRenderCtx,
		comps: &'comps World,
	) {
		if render_ctx.zsorted_children_list.is_empty() {
			// Optimization: Nothing to be drawn, skip context switching
			return;
		}

		self.begin_composite_content(as_mask, drawable, composite, render_ctx);

		for uuid in &render_ctx.zsorted_children_list {
			let drawable_kind =
				DrawableKind::new(*uuid, comps).expect("All children in zsorted_children_list should be a Drawable.");
			match drawable_kind {
				DrawableKind::TexturedMesh {
					transform,
					drawable,
					data,
				} => self.draw_textured_mesh_content(
					as_mask,
					camera,
					transform, // Note: this is already the absolute transform, no need to multiply again
					drawable,
					data,
					comps.get(*uuid).unwrap(),
				),
				DrawableKind::Composite { .. } => panic!("Composite inside Composite not allowed."),
			}
		}

		self.finish_composite_content(as_mask, drawable, composite, render_ctx);
	}

	fn draw(&self, camera: &Mat4, puppet: &Puppet) {
		for uuid in &puppet
			.render_ctx
			.as_ref()
			.expect("RenderCtx of puppet must be initialized before calling draw().")
			.root_drawables_zsorted
		{
			let drawable_kind = DrawableKind::new(*uuid, &puppet.node_comps)
				.expect("Every node in root_drawables_zsorted must be a Drawable.");
			self.draw_drawable(false, camera, &drawable_kind, &puppet.node_comps, *uuid);
		}
	}
}
