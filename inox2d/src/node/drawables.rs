use crate::math::transform::Transform;
use crate::node::{
	components::{Composite, Drawable, TexturedMesh},
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
	pub(crate) fn new(id: InoxNodeUuid, comps: &'comps World) -> Option<Self> {
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
