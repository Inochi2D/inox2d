use crate::puppet::Puppet;

#[derive(Clone, Debug)]
pub struct Texture {
    pub format: image::ImageFormat,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Model {
    pub puppet: Puppet,
    pub textures: Vec<Texture>,
}
