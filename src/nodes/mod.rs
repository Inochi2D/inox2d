use serde::Serialize;

use self::node::Node;

pub mod node;
pub mod drivers;

pub mod composite;
pub mod drawable;
pub mod part;

#[derive(Debug, Default)]
pub struct NodeTree<S: Serialize>(Vec<Box<dyn Node<S>>>);