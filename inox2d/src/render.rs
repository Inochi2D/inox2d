mod vertex_buffers;


use crate::math::transform::Transform;
use crate::node::{
	components::{Composite, Drawable, TexturedMesh},
	InoxNodeUuid,
};
use crate::puppet::{InoxNodeTree, World};

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

struct TexturedMeshRenderCtx {
	pub index_offset: u16,
	pub vert_offset: u16,
	pub index_len: usize,
	pub vert_len: usize,
}

struct CompositeRenderCtx {
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
						// if any of the children is not a drawable, we have a problem, but it will error later
						let mut zsorted_children_list: Vec<InoxNodeUuid> =
							nodes.get_children(node.uuid).map(|n| n.uuid).collect();
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

/*
use crate::mesh::Mesh;
use crate::model::Model;
use crate::node::data::{Composite, InoxData, MaskMode, Part};
use crate::puppet::Puppet;


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
	/// The render pass.
	///
	/// Logical error if this puppet is not from the latest prepared model.
	fn render(&self, puppet: &Puppet);
	/// Finish one render pass.
	fn on_end_scene(&self);
	/// Actually make results visible, e.g. on a screen/texture.
	fn draw_scene(&self);

	/// Begin masking.
	///
	/// Clear and start writing to the stencil buffer, lock the color buffer.
	fn on_begin_mask(&self, has_mask: bool);
	/// The following draw calls consist of a mask or dodge mask.
	fn set_mask_mode(&self, dodge: bool);
	/// Read only from the stencil buffer, unlock the color buffer.
	fn on_begin_masked_content(&self);
	/// Disable the stencil buffer.
	fn on_end_mask(&self);

	/// Draw contents of a mesh-defined plain region.
	// TODO: plain mesh (usually for mesh masks) not implemented
	fn draw_mesh_self(&self, as_mask: bool, camera: &Mat4);

	/// Draw contents of a part.
	// TODO: Merging of Part and PartRenderCtx?
	// TODO: Inclusion of NodeRenderCtx into Part?
	fn draw_part_self(
		&self,
		as_mask: bool,
		camera: &Mat4,
		node_render_ctx: &NodeRenderCtx,
		part: &Part,
		part_render_ctx: &PartRenderCtx,
	);

	/// When something needs to happen before drawing to the composite buffers.
	fn begin_composite_content(&self);
	/// Transfer content from composite buffers to normal buffers.
	fn finish_composite_content(&self, as_mask: bool, composite: &Composite);
}

pub trait InoxRendererCommon {
	/// Draw one part, with its content properly masked.
	fn draw_part(
		&self,
		camera: &Mat4,
		node_render_ctx: &NodeRenderCtx,
		part: &Part,
		part_render_ctx: &PartRenderCtx,
		puppet: &Puppet,
	);

	/// Draw one composite.
	fn draw_composite(
		&self,
		as_mask: bool,
		camera: &Mat4,
		composite: &Composite,
		puppet: &Puppet,
		children: &[InoxNodeUuid],
	);

	/// Iterate over top-level drawables (excluding masks) in zsort order,
	/// and make draw calls correspondingly.
	///
	/// This effectively draws the complete puppet.
	fn draw(&self, camera: &Mat4, puppet: &Puppet);
}

impl<T: InoxRenderer> InoxRendererCommon for T {
	fn draw_part(
		&self,
		camera: &Mat4,
		node_render_ctx: &NodeRenderCtx,
		part: &Part,
		part_render_ctx: &PartRenderCtx,
		puppet: &Puppet,
	) {
		let masks = &part.draw_state.masks;
		if !masks.is_empty() {
			self.on_begin_mask(part.draw_state.has_masks());
			for mask in &part.draw_state.masks {
				self.set_mask_mode(mask.mode == MaskMode::Dodge);

				let mask_node = puppet.nodes.get_node(mask.source).unwrap();
				let mask_node_render_ctx = &puppet.render_ctx.node_render_ctxs[&mask.source];

				match (&mask_node.data, &mask_node_render_ctx.kind) {
					(InoxData::Part(ref mask_part), RenderCtxKind::Part(ref mask_part_render_ctx)) => {
						self.draw_part_self(true, camera, mask_node_render_ctx, mask_part, mask_part_render_ctx);
					}

					(InoxData::Composite(ref mask_composite), RenderCtxKind::Composite(ref mask_children)) => {
						self.draw_composite(true, camera, mask_composite, puppet, mask_children);
					}

					_ => {
						// This match block clearly is sign that the data structure needs rework
						todo!();
					}
				}
			}
			self.on_begin_masked_content();
			self.draw_part_self(false, camera, node_render_ctx, part, part_render_ctx);
			self.on_end_mask();
		} else {
			self.draw_part_self(false, camera, node_render_ctx, part, part_render_ctx);
		}
	}

	fn draw_composite(
		&self,
		as_mask: bool,
		camera: &Mat4,
		comp: &Composite,
		puppet: &Puppet,
		children: &[InoxNodeUuid],
	) {
		if children.is_empty() {
			// Optimization: Nothing to be drawn, skip context switching
			return;
		}

		self.begin_composite_content();

		for &uuid in children {
			let node = puppet.nodes.get_node(uuid).unwrap();
			let node_render_ctx = &puppet.render_ctx.node_render_ctxs[&uuid];

			if let (InoxData::Part(ref part), RenderCtxKind::Part(ref part_render_ctx)) =
				(&node.data, &node_render_ctx.kind)
			{
				if as_mask {
					self.draw_part_self(true, camera, node_render_ctx, part, part_render_ctx);
				} else {
					self.draw_part(camera, node_render_ctx, part, part_render_ctx, puppet);
				}
			} else {
				// composite inside composite simply cannot happen
			}
		}

		self.finish_composite_content(as_mask, comp);
	}

	fn draw(&self, camera: &Mat4, puppet: &Puppet) {
		for &uuid in &puppet.render_ctx.root_drawables_zsorted {
			let node = puppet.nodes.get_node(uuid).unwrap();
			let node_render_ctx = &puppet.render_ctx.node_render_ctxs[&uuid];

			match (&node.data, &node_render_ctx.kind) {
				(InoxData::Part(ref part), RenderCtxKind::Part(ref part_render_ctx)) => {
					self.draw_part(camera, node_render_ctx, part, part_render_ctx, puppet);
				}

				(InoxData::Composite(ref composite), RenderCtxKind::Composite(ref children)) => {
					self.draw_composite(false, camera, composite, puppet, children);
				}

				_ => {
					// This clearly is sign that the data structure needs rework
					todo!();
				}
			}
		}
	}
}
*/
