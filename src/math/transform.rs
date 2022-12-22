use std::ops::Mul;

use glam::{EulerRot, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

#[derive(Clone, Debug)]
pub struct Transform {
    trs: Mat4,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec2,
    /// Whether the transform should snap to pixels
    pub pixel_snap: bool,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            trs: Mat4::IDENTITY,
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec2::ONE,
            pixel_snap: false,
        }
    }
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_pixel_snap(mut self, pixel_snap: bool) -> Self {
        self.pixel_snap = pixel_snap;
        self
    }

    pub fn matrix(&self) -> Mat4 {
        self.trs
    }

    /// Update the internal matrix
    pub fn update(&mut self) {
        self.trs = Mat4::from_translation(self.translation)
            * Mat4::from_quat(Quat::from_euler(
                EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ))
            * Mat4::from_scale(Vec3::new(self.scale.x, self.scale.y, 1.));
    }

    /// Resets transformation, rotation and scale
    pub fn clear(&mut self) {
        self.translation = Vec3::ZERO;
        self.rotation = Vec3::ZERO;
        self.scale = Vec2::ONE;
    }

    pub fn calc_offset(&self, other: &Self) -> Self {
        Self::new()
            .with_translation(self.translation + other.translation)
            .with_rotation(self.rotation + other.rotation)
            .with_scale(self.scale + other.scale)
    }
}

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let strs = self.trs * rhs.trs;
        let trans = strs * Vec4::ONE;
        Self {
            trs: strs,
            translation: Vec3::new(trans.x, trans.y, trans.z),
            rotation: self.rotation * rhs.rotation,
            scale: self.scale * rhs.scale,
            pixel_snap: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Transform2D {
    trs: Mat3,
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            trs: Mat3::IDENTITY,
            translation: Vec2::ZERO,
            rotation: 0.,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    pub fn matrix(&self) -> Mat3 {
        self.trs
    }

    pub fn update(&mut self) {
        self.trs = Mat3::from_translation(self.translation)
            * Mat3::from_rotation_z(self.rotation)
            * Mat3::from_scale(self.scale);
    }
}
