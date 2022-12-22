use crate::puppet::ExtPuppet;

#[derive(Clone, Debug)]
pub struct ModelTexture {
    pub format: image::ImageFormat,
    pub data: Vec<u8>,
}

pub type Model = ExtModel<()>;

#[derive(Debug)]
pub struct ExtModel<T> {
    pub puppet: ExtPuppet<T>,
    pub textures: Vec<ModelTexture>,
}
