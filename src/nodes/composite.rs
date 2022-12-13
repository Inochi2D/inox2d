use serde::{Deserialize, Serialize};

use super::drawable::Drawable;
use super::node::{Node, NodeState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    #[serde(flatten)]
    pub(crate) node_state: NodeState,
    #[serde(flatten)]
    pub(crate) draw_state: Drawable,
}

#[typetag::serde]
impl Node for Composite {
    fn get_node_state(&self) -> &NodeState {
        &self.node_state
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        &mut self.node_state
    }
}
