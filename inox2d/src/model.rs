use std::fmt;
use std::sync::Arc;

use crate::puppet::Puppet;

#[derive(Clone, Debug)]
pub struct ModelTexture {
	pub format: image::ImageFormat,
	pub data: Arc<[u8]>,
}

#[derive(Clone, Debug)]
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
		writeln!(f, "{name} {}", json::stringify_pretty(self.payload.clone(), 2))
	}
}

/// Inochi2D model.
pub struct Model {
	pub puppet: Puppet,
	pub textures: Vec<ModelTexture>,
	pub vendors: Vec<VendorData>,
}
