pub mod drivers;
pub mod node;
pub mod node_tree;

pub mod composite;
pub mod drawable;
pub mod part;

#[macro_export]
macro_rules! impl_node {
    ($t:ty, $field:ident) => {
        // #[typetag::serde]
        impl $crate::nodes::node::Node for $t {
            fn get_node_state(&self) -> &$crate::nodes::node::NodeState {
                &self.$field
            }

            fn get_node_state_mut(&mut self) -> &mut $crate::nodes::node::NodeState {
                &mut self.$field
            }

            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {
                self
            }
        }
    };
}
