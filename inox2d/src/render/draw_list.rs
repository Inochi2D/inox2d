use std::cell::RefCell;

use crate::node::{
	components::{Mask, Masks},
	drawables::{CompositeComponents, TexturedMeshComponents},
	InoxNodeUuid,
};
use crate::render::{CompositeRenderCtx, InoxRenderer, TexturedMeshRenderCtx};

#[derive(Clone)]
pub enum DrawCommand<'a> {
	BeginMasks(&'a Masks),
	BeginMask(&'a Mask),
	BeginMaskedContent,
	EndMask,
	DrawTexturedMesh {
		as_mask: bool,
		components: TexturedMeshComponents<'a>,
		render_ctx: &'a TexturedMeshRenderCtx,
		id: InoxNodeUuid,
	},
	BeginComposite {
		as_mask: bool,
		components: CompositeComponents<'a>,
		render_ctx: &'a CompositeRenderCtx,
		id: InoxNodeUuid,
	},
	FinishComposite {
		as_mask: bool,
		components: CompositeComponents<'a>,
		render_ctx: &'a CompositeRenderCtx,
		id: InoxNodeUuid,
	},
}

#[derive(Default)]
pub struct DrawList<'a> {
	pub commands: RefCell<Vec<DrawCommand<'a>>>,
}

impl<'a> DrawList<'a> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn clear(&self) {
		self.commands.borrow_mut().clear();
	}
}

impl<'a> InoxRenderer<'a> for DrawList<'a> {
	fn on_begin_masks(&self, masks: &'a Masks) {
		self.commands.borrow_mut().push(DrawCommand::BeginMasks(masks));
	}

	fn on_begin_mask(&self, mask: &'a Mask) {
		self.commands.borrow_mut().push(DrawCommand::BeginMask(mask));
	}

	fn on_begin_masked_content(&self) {
		self.commands.borrow_mut().push(DrawCommand::BeginMaskedContent);
	}

	fn on_end_mask(&self) {
		self.commands.borrow_mut().push(DrawCommand::EndMask);
	}

	fn draw_textured_mesh_content(
		&self,
		as_mask: bool,
		components: TexturedMeshComponents<'a>,
		render_ctx: &'a TexturedMeshRenderCtx,
		id: InoxNodeUuid,
	) {
		self.commands.borrow_mut().push(DrawCommand::DrawTexturedMesh {
			as_mask,
			components,
			render_ctx,
			id,
		});
	}

	fn begin_composite_content(
		&self,
		as_mask: bool,
		components: CompositeComponents<'a>,
		render_ctx: &'a CompositeRenderCtx,
		id: InoxNodeUuid,
	) {
		self.commands.borrow_mut().push(DrawCommand::BeginComposite {
			as_mask,
			components,
			render_ctx,
			id,
		});
	}

	fn finish_composite_content(
		&self,
		as_mask: bool,
		components: CompositeComponents<'a>,
		render_ctx: &'a CompositeRenderCtx,
		id: InoxNodeUuid,
	) {
		self.commands.borrow_mut().push(DrawCommand::FinishComposite {
			as_mask,
			components,
			render_ctx,
			id,
		});
	}
}
