#[cfg(feature = "opengl")]
pub mod opengl;

pub trait App
where
    Self::Error: std::error::Error,
{
    fn update(&self, event: winit::event::Event<()>);
    fn launch(window: &winit::window::Window) -> Result<Self, Self::Error>
    where
        Self: Sized;
    type Error;
}
