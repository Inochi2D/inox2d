use glam::{Mat4, Vec2, Vec3};

pub struct Camera {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2{ x: 0.0, y: 0.0 },
            rotation: 0.0,
            scale: Vec2::ONE
        }
    }
}

impl Camera {

    /// Gets the real size of the viewport
    pub fn get_real_size(&mut self, viewport: Vec2) -> Vec2 {
        Vec2 {
            x: viewport.x/self.scale.x, 
            y: viewport.y/self.scale.y,
        }
    }

    /// Gets the center offset of the viewport
    pub fn get_center_offset(&mut self, viewport: Vec2) -> Vec2 {
        let real_size = self.get_real_size(viewport);
        Vec2 { x: (real_size.x/2.0), y: (real_size.y/2.0) }
    }

    /// Gets the resulting matrix from the camera and viewport
    pub fn matrix(&mut self, viewport: Vec2) -> Mat4 {
        let real_size = self.get_real_size(viewport);

        // Faster to reuse real_size, so do that instead of calling get_center_offset
        let origin = Vec2{ x: real_size.x/2.0, y: real_size.y/2.0 };
        let pos = Vec3{ x: self.position.x, y: self.position.y, z: -(u16::MAX as f32)/2.0};

        // Return camera ortho matrix
        Mat4::orthographic_lh(0.0, real_size.x, real_size.y, 0.0, 0.0, u16::MAX as f32) * 
            Mat4::from_translation(Vec3{x: origin.x, y: origin.y, z: 0.0}) *
            Mat4::from_rotation_z(self.rotation) *
            Mat4::from_translation(pos)
    }
}