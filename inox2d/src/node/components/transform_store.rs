use glam::Mat4;

use crate::math::transform::TransformOffset;

#[derive(Default)]
pub(crate) struct TransformStore {
	pub absolute: Mat4,
	pub relative: TransformOffset,
}
