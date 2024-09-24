use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};

/// relative transform
#[derive(Debug, Clone)]
pub struct TransformOffset {
	/// X Y Z
	pub translation: Vec3,
	/// Euler angles
	pub rotation: Vec3,
	/// X Y zoom
	pub scale: Vec2,
	/// Whether the transform should snap to pixels
	pub pixel_snap: bool,
}

impl Default for TransformOffset {
	fn default() -> Self {
		Self {
			translation: Vec3::ZERO,
			rotation: Vec3::ZERO,
			scale: Vec2::ONE,
			pixel_snap: false,
		}
	}
}

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
