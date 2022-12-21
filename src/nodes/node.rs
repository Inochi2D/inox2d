use std::fmt::Debug;

use crate::math::transform::Transform;

use super::composite::Composite;
use super::drivers::simple_physics::SimplePhysics;
use super::part::Part;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InoxNodeUuid(pub(crate) u32);

#[derive(Debug, Clone)]
pub enum InoxData<T> {
    Node,
    Part(Part),
    Composite(Composite),
    SimplePhysics(SimplePhysics),
    Custom(T),
}

impl<T> InoxData<T> {
    pub fn is_node(&self) -> bool {
        matches!(self, InoxData::Node)
    }

    pub fn is_part(&self) -> bool {
        matches!(self, InoxData::Part(_))
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, InoxData::Composite(_))
    }

    pub fn is_simple_physics(&self) -> bool {
        matches!(self, InoxData::SimplePhysics(_))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, InoxData::Custom(_))
    }

    pub fn data_type_name(&self) -> &'static str {
        match self {
            InoxData::Node => "Node",
            InoxData::Part(_) => "Part",
            InoxData::Composite(_) => "Composite",
            InoxData::SimplePhysics(_) => "SimplePhysics",
            InoxData::Custom(_) => "Custom",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InoxNode<T> {
    pub uuid: InoxNodeUuid,
    pub name: String,
    pub enabled: bool,
    pub zsort: f32,
    pub transform: Transform,
    pub lock_to_root: bool,
    pub data: InoxData<T>,
}

impl<T> InoxNode<T> {
    pub fn is_node(&self) -> bool {
        self.data.is_node()
    }

    pub fn is_part(&self) -> bool {
        self.data.is_part()
    }

    pub fn is_composite(&self) -> bool {
        self.data.is_composite()
    }

    pub fn is_simple_physics(&self) -> bool {
        self.data.is_simple_physics()
    }

    pub fn is_custom(&self) -> bool {
        self.data.is_custom()
    }

    pub fn node_type_name(&self) -> &'static str {
        self.data.data_type_name()
    }
}
