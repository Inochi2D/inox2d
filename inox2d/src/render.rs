use std::collections::HashMap;

use glam::{vec2, Mat4, Vec2};

use crate::math::transform::TransformOffset;
use crate::mesh::Mesh;
use crate::model::Model;
use crate::node::data::{Composite, InoxData, MaskMode, Part};
use crate::node::InoxNodeUuid;
use crate::puppet::Puppet;

#[derive(Clone, Debug)]
pub struct VertexBuffers {
	pub verts: Vec<Vec2>,
	pub uvs: Vec<Vec2>,
	pub indices: Vec<u16>,
	pub deforms: Vec<Vec2>,
}

impl Default for VertexBuffers {
	fn default() -> Self {
		// init with a quad covering the whole viewport

		#[rustfmt::skip]
        let verts = vec![
            vec2(-1.0, -1.0),
            vec2(-1.0,  1.0),
            vec2( 1.0, -1.0),
            vec2( 1.0,  1.0),
        ];

		#[rustfmt::skip]
        let uvs = vec![
            vec2(0.0, 0.0),
            vec2(0.0, 1.0),
            vec2(1.0, 0.0),
            vec2(1.0, 1.0),
        ];

		#[rustfmt::skip]
        let indices = vec![
            0, 1, 2,
            2, 1, 3,
        ];

		let deforms = vec![Vec2::ZERO; 4];

		Self {
			verts,
			uvs,
			indices,
			deforms,
		}
	}
}

impl VertexBuffers {
	/// Adds the mesh's vertices and UVs to the buffers and returns its index and vertex offset.
	pub fn push(&mut self, mesh: &Mesh) -> (u16, u16) {
		let index_offset = self.indices.len() as u16;
		let vert_offset = self.verts.len() as u16;

		self.verts.extend_from_slice(&mesh.vertices);
		self.uvs.extend_from_slice(&mesh.uvs);
		self.indices
			.extend(mesh.indices.iter().map(|index| index + vert_offset));
		self.deforms
			.resize(self.deforms.len() + mesh.vertices.len(), Vec2::ZERO);

		(index_offset, vert_offset)
	}
}

#[derive(Debug, Clone)]
pub struct PartRenderCtx {
	pub index_offset: u16,
	pub vert_offset: u16,
	pub index_len: usize,
	pub vert_len: usize,
}

#[derive(Debug, Clone)]
pub enum RenderCtxKind {
	Node,
	Part(PartRenderCtx),
	Composite(Vec<InoxNodeUuid>),
}

#[derive(Clone, Debug)]
pub struct NodeRenderCtx {
	pub trans: Mat4,
	pub trans_offset: TransformOffset,
	pub kind: RenderCtxKind,
}

pub type NodeRenderCtxs = HashMap<InoxNodeUuid, NodeRenderCtx>;

#[derive(Clone, Debug)]
pub struct RenderCtx {
	pub vertex_buffers: VertexBuffers,
	/// All nodes that need respective draw method calls:
	/// - including standalone parts and composite parents,
	/// - excluding plain mesh masks and composite children.
	pub root_drawables_zsorted: Vec<InoxNodeUuid>,
	pub node_render_ctxs: NodeRenderCtxs,
}

impl RenderCtx {
	pub fn new<T>(nodes: &InoxNodeTree<T>) -> Self {
		let mut vertex_buffers = VertexBuffers::default();
		let mut root_drawables_zsorted: Vec<InoxNodeUuid> = Vec::new();
		let mut node_render_ctxs = HashMap::new();

		for uuid in nodes.all_node_ids() {
			let node = nodes.get_node(uuid).unwrap();

			node_render_ctxs.insert(
				uuid,
				NodeRenderCtx {
					trans: Mat4::default(),
					trans_offset: node.trans_offset,
					kind: match node.data {
						InoxData::Part(ref part) => {
							let (index_offset, vert_offset) = vertex_buffers.push(&part.mesh);
							RenderCtxKind::Part(PartRenderCtx {
								index_offset,
								vert_offset,
								index_len: part.mesh.indices.len(),
								vert_len: part.mesh.vertices.len(),
							})
						}

						InoxData::Composite(_) => RenderCtxKind::Composite(nodes.zsorted_children(uuid)),

						_ => RenderCtxKind::Node,
					},
				},
			);
		}

		for uuid in nodes.zsorted_root() {
			let node = nodes.get_node(uuid).unwrap();

			match node.data {
				InoxData::Part(_) | InoxData::Composite(_) => {
					root_drawables_zsorted.push(uuid);
				}

				_ => (),
			}
		}

		Self {
			vertex_buffers,
			root_drawables_zsorted,
			node_render_ctxs,
		}
	}
}

impl Puppet {
	/// Update the puppet's nodes' absolute transforms, by combining transforms
	/// from each node's ancestors in a pre-order traversal manner.
	pub fn update_trans(&mut self) {
		let root_node = self.nodes.arena[self.nodes.root].get();
		let node_rctxs = &mut self.render_ctx.node_render_ctxs;

		// The root's absolute transform is its relative transform.
		let root_trans = node_rctxs.get(&root_node.uuid).unwrap().trans_offset.to_matrix();

		// Pre-order traversal, just the order to ensure that parents are accessed earlier than children
		// Skip the root
		for id in self.nodes.root.descendants(&self.nodes.arena).skip(1) {
			let node_index = &self.nodes.arena[id];
			let node = node_index.get();

			if node.lock_to_root {
				let node_render_ctx = node_rctxs.get_mut(&node.uuid).unwrap();
				node_render_ctx.trans = root_trans * node_render_ctx.trans_offset.to_matrix();
			} else {
				let parent = &self.nodes.arena[node_index.parent().unwrap()].get();
				let parent_trans = node_rctxs.get(&parent.uuid).unwrap().trans;

				let node_render_ctx = node_rctxs.get_mut(&node.uuid).unwrap();
				node_render_ctx.trans = parent_trans * node_render_ctx.trans_offset.to_matrix();
			}
		}
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
