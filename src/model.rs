use crate::puppet::ExtPuppet;
use crate::texture::CompressedTexture;

/// Inochi2D model.
pub type Model = ExtModel<()>;

/// Extensible Inochi2D model.
#[derive(Debug)]
pub struct ExtModel<T> {
    pub puppet: ExtPuppet<T>,
    pub textures: Vec<CompressedTexture>,
}
