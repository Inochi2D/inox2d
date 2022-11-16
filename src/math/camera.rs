use glam::{Mat4, Vec2, Vec3};

use crate::core::in_get_viewport;

/// An orthographic camera
#[derive(Clone, Debug, Default)]
pub struct Camera {
    // projection: Mat4,
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Camera {
    pub fn real_size(&self) -> Vec2 {
        let (width, height) = in_get_viewport();
        Vec2::new(width as f32 / self.scale.x, height as f32 / self.scale.y)
    }

    pub fn center_offset(&self) -> Vec2 {
        self.real_size() / 2.
    }

    pub fn matrix(&mut self) -> Mat4 {
        if !self.position.is_finite() {
            self.position = Vec2::ZERO;
        }
        if !self.scale.is_finite() {
            self.scale = Vec2::ONE;
        }
        if !self.rotation.is_finite() {
            self.rotation = 0.;
        };

        let real_size = self.real_size();
        if !real_size.is_finite() {
            return Mat4::IDENTITY;
        }

        let origin = real_size / 2.;
        let position = Vec3::new(self.position.x, self.position.y, -(u16::MAX as f32 / 2.));

        // TODO: which orthographic is it?
        Mat4::orthographic_lh(0., real_size.x, real_size.y, 0., 0., u16::MAX as f32)
            * Mat4::from_translation(Vec3::new(origin.x, origin.y, 0.))
            * Mat4::from_rotation_z(self.rotation)
            * Mat4::from_translation(position)
    }
}
