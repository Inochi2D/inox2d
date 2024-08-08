use glam::Mat4;

use crate::node::{
	components::{Composite, Drawable, Mesh, TexturedMesh, TransformStore},
	InoxNodeUuid,
};
use crate::puppet::World;

/// Possible component combinations of a renderable node.
///
/// Future spec extensions go here.
/// For user-defined custom nodes that can be rendered, as long as a subset of their components matches one of these variants,
/// they will be picked up and enter the regular rendering pipeline.
pub(crate) enum DrawableKind<'comps> {
	TexturedMesh(TexturedMeshComponents<'comps>),
	Composite(CompositeComponents<'comps>),
}

/// Pack of components for a TexturedMesh. "Part" in Inochi2D terms.
pub struct TexturedMeshComponents<'comps> {
	// Only the absolute part of `TransformStore` that the renderer backend may need.
	pub transform: &'comps Mat4,
	pub drawable: &'comps Drawable,
	pub texture: &'comps TexturedMesh,
	pub mesh: &'comps Mesh,
}

/// Pack of components for a Composite node.
pub struct CompositeComponents<'comps> {
	// Only the absolute part of `TransformStore` that the renderer backend may need.
	pub transform: &'comps Mat4,
	pub drawable: &'comps Drawable,
	pub data: &'comps Composite,
}

impl<'comps> DrawableKind<'comps> {
	/// Tries to construct a renderable node data pack from the World of components.
	/// `None` if node not renderable.
	///
	/// If `check`, will send a warning to `tracing` if component combination non-standard for a supposed-to-be Drawable node.
	pub(crate) fn new(id: InoxNodeUuid, comps: &'comps World, check: bool) -> Option<Self> {
		let drawable = match comps.get::<Drawable>(id) {
			Some(drawable) => drawable,
			None => return None,
		};
		let transform = &comps
			.get::<TransformStore>(id)
			.expect("A drawble must have an associated transform.")
			.absolute;
		let textured_mesh = comps.get::<TexturedMesh>(id);
		let composite = comps.get::<Composite>(id);

		match (textured_mesh.is_some(), composite.is_some()) {
			(true, true) => {
				if check {
					tracing::warn!(
						"Node {} as a Drawable has both TexturedMesh and Composite, treat as TexturedMesh.",
						id.0
					);
				}
				Some(DrawableKind::TexturedMesh(TexturedMeshComponents {
					transform,
					drawable,
					texture: textured_mesh.unwrap(),
					mesh: comps
						.get::<Mesh>(id)
						.expect("A TexturedMesh must have an associated Mesh."),
				}))
			}
			(false, false) => {
				if check {
					tracing::warn!(
						"Node {} as a Drawable has neither TexturedMesh nor Composite, skipping.",
						id.0
					);
				}
				None
			}
			(true, false) => Some(DrawableKind::TexturedMesh(TexturedMeshComponents {
				transform,
				drawable,
				texture: textured_mesh.unwrap(),
				mesh: comps
					.get::<Mesh>(id)
					.expect("A TexturedMesh must have an associated Mesh."),
			})),
			(false, true) => Some(DrawableKind::Composite(CompositeComponents {
				transform,
				drawable,
				data: composite.unwrap(),
			})),
		}
	}
}
