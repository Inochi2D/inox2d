use std::fmt::Debug;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::math::transform::Transform;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct NodeId(pub(crate) usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    #[serde(skip)]
    id: NodeId,
    #[serde(skip)]
    parent: Option<NodeId>,
    #[serde(skip)]
    children: Vec<NodeId>,
    uuid: u32,
    name: String,
    enabled: bool,
    zsort: f32,
    transform: Transform,
    #[serde(rename = "lockToRoot")]
    lock_to_root: bool,
}

// TODO: make a derive macro for this
pub trait Node<S: Serializer>: Debug {
    fn get_node_state(&self) -> &NodeState;
    fn get_node_state_mut(&mut self) -> &mut NodeState;
    fn serialize_node(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

// TODO: make a derive macro for this
pub trait NodeDeserializer<'de, D: Deserializer<'de>, S: Serializer> {
    const NODE_TYPE: &'static str;

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node<S>>, D::Error>;
}

impl<S: Serializer> Node<S> for NodeState {
    fn get_node_state(&self) -> &NodeState {
        self
    }

    fn get_node_state_mut(&mut self) -> &mut NodeState {
        self
    }

    fn serialize_node(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.serialize(serializer)
    }
}

impl<'de, D, S> NodeDeserializer<'de, D, S> for NodeState
where
    D: Deserializer<'de>,
    S: Serializer,
{
    const NODE_TYPE: &'static str = "Node";

    fn deserialize_node(&self, deserializer: D) -> Result<Box<dyn Node<S>>, D::Error> {
        let part: Self = Self::deserialize(deserializer)?;
        Ok(Box::new(part))
    }
}