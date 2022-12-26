use crate::{model::ModelTexture, nodes::node_tree::ExtInoxNodeTree};

#[cfg(feature = "opengl")]
pub mod opengl;

pub trait App
where
    Self::Error: std::error::Error,
{
    fn update(&self, event: winit::event::Event<()>);
    fn launch(
        window: &winit::window::Window,
        nodes: ExtInoxNodeTree<Self::NodeData>,
        textures: Vec<ModelTexture>,
        custom_renderer: Self::CustomRenderer,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized;
    type Error;
    type NodeData;
    type CustomRenderer;
}
