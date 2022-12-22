use crate::{nodes::node_tree::NodeTree, model::ModelTexture};

#[cfg(feature = "opengl")]
pub mod opengl;

#[cfg(feature = "wgpu")]
pub mod wgpu;

pub trait App
where
    Self::Error: std::error::Error,
{
    fn update(&self, event: winit::event::Event<()>);
    fn launch(window: &winit::window::Window, nodes: NodeTree, textures: Vec<ModelTexture>) -> Result<Self, Self::Error>
    where
        Self: Sized;
    type Error;
}
