use serde::{Deserialize, Serialize};

use crate::mesh::Mesh;

use super::drawable::Drawable;
use super::node::{Node, NodeState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(flatten)]
    pub node_state: NodeState,
    #[serde(flatten)]
    pub draw_state: Drawable,
    pub mesh: Mesh,
    pub textures: [usize; 3],
    #[cfg(feature = "opengl")]
    #[serde(skip)]
    pub start_indice: u16,
    // start_deform: u16,
}

#[typetag::serde]
impl Node for Part {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }
}

impl Part {
    pub(crate) fn num_indices(&self) -> u16 {
        self.mesh.indices.len() as u16
    }
}
