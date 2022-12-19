use crate::impl_node;

use super::drawable::Drawable;
use super::node::NodeState;

#[derive(Debug, Clone)]
pub struct Composite {
    pub(crate) node_state: NodeState,
    pub(crate) draw_state: Drawable,
}

impl_node!(Composite, node_state);