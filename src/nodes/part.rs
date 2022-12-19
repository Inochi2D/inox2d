use crate::impl_node;
use crate::mesh::Mesh;

use super::drawable::Drawable;
use super::node::NodeState;

#[derive(Debug, Clone)]
pub struct Part {
    pub node_state: NodeState,
    pub draw_state: Drawable,
    pub mesh: Mesh,
    pub textures: [usize; 3],
    #[cfg(feature = "opengl")]
    pub start_indice: u16,
    // start_deform: u16,
}

impl_node!(Part, node_state);

impl Part {
    pub(crate) fn num_indices(&self) -> u16 {
        self.mesh.indices.len() as u16
    }
}
