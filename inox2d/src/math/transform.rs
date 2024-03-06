use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::node::data::TransformOffset;

impl TransformOffset {
	pub fn to_matrix(&self) -> Mat4 {
		Mat4::from_translation(self.translation)
			* Mat4::from_quat(Quat::from_euler(
				EulerRot::XYZ,
				self.rotation.x,
				self.rotation.y,
				self.rotation.z,
			)) * Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.))
	}
}
