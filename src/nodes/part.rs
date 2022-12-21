use crate::mesh::Mesh;

use super::drawable::Drawable;

#[derive(Debug, Clone)]
pub struct Part {
    pub draw_state: Drawable,
    pub mesh: Mesh,
    pub tex_albedo: usize,
    pub tex_emissive: usize,
    pub tex_bumpmap: usize,
    #[cfg(feature = "opengl")]
    pub start_indice: u16,
    // start_deform: u16,
}

impl Part {
    pub fn num_indices(&self) -> u16 {
        self.mesh.indices.len() as u16
    }
}
