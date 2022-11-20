use serde::{Deserialize, Serialize};

use super::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTree {
    #[serde(flatten)]
    node: Box<dyn Node>,
    #[serde(default)]
    children: Vec<NodeTree>,
}
