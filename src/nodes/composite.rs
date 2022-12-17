use crate::impl_node;

use serde::{Deserialize, Serialize};

use super::drawable::Drawable;
use super::node::NodeState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composite {
    #[serde(flatten)]
    pub(crate) node_state: NodeState,
    #[serde(flatten)]
    pub(crate) draw_state: Drawable,
}

impl_node!(Composite, node_state);