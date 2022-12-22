use glam::Vec2;

#[derive(Debug, Clone)]
pub struct SimplePhysics {
    pub param: u32,
    pub model_type: String,
    pub map_mode: String,
    pub gravity: f32,
    pub length: f32,
    pub frequency: f32,
    pub angle_damping: f32,
    pub length_damping: f32,
    pub output_scale: Vec2,
}
