use glam::Mat4;

use crate::math::transform::TransformOffset;

/// Component holding transform values that may be modified across frames.
#[derive(Default)]
pub(crate) struct TransformStore {
	pub absolute: Mat4,
	pub relative: TransformOffset,
}
