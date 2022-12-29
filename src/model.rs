use crate::puppet::ExtPuppet;

/// Texture of a model.
#[derive(Clone, Debug)]
pub struct ModelTexture {
    pub format: image::ImageFormat,
    pub data: Vec<u8>,
}

/// Inochi2D model.
pub type Model = ExtModel<()>;

/// Extensible Inochi2D model.
#[derive(Debug)]
pub struct ExtModel<T> {
    pub puppet: ExtPuppet<T>,
    pub textures: Vec<ModelTexture>,
}
