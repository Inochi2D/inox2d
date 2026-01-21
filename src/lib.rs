#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use std::fs::File;
use std::io::BufReader;
use napi::bindgen_prelude::*;
use inox2d::formats::inp::parse_inp;

#[napi]
pub struct InoxModel {
  pub(crate) inner: inox2d::model::Model,
}

#[napi]
impl InoxModel {
  #[napi(factory)]
  pub fn from_path(path: String) -> Result<InoxModel> {
    let file = File::open(&path).map_err(|e| Error::from_reason(e.to_string()))?;
    let reader = BufReader::new(file);
    let model = parse_inp(reader)
        .map_err(|e| Error::from_reason(format!("Failed to parse INP: {}", e)))?;

    Ok(InoxModel { inner: model })
  }

  #[napi(getter)]
  pub fn get_texture_count(&self) -> u32 {
    self.inner.textures.len() as u32
  }

  #[napi(getter)]
  pub fn get_vendors(&self) -> Vec<String> {
      self.inner.vendors.iter().map(|v| v.name.clone()).collect()
  }
}
