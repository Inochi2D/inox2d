use std::fmt;

use crate::puppet::Puppet;

#[derive(Debug)]
pub struct ModelTexture {
    pub format: image::ImageFormat,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct VendorData {
    pub name: String,
    pub payload: json::JsonValue,
}

impl fmt::Display for VendorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        #[cfg(feature = "owo")]
        let name = {
            use owo_colors::OwoColorize;
            name.green()
        };
        writeln!(
            f,
            "{name} {}",
            json::stringify_pretty(self.payload.clone(), 2)
        )
    }
}

/// Inochi2D model.
#[derive(Debug)]
pub struct Model<T = ()> {
    pub puppet: Puppet<T>,
    pub textures: Vec<ModelTexture>,
    pub vendors: Vec<VendorData>,
}
