use std::ops::{Deref, DerefMut};

use glow::HasContext;

pub struct GlBuffer<T>(Vec<T>);

impl<T> Deref for GlBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for GlBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> GlBuffer<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from(buffer: Vec<T>) -> Self {
        Self(buffer)
    }

    pub fn upload(&self, gl: &glow::Context, target: u32, usage: u32) -> glow::NativeBuffer {
        let slice = self.as_slice();
        unsafe {
            let bytes: &[u8] = core::slice::from_raw_parts(
                slice.as_ptr() as *const u8,
                slice.len() * core::mem::size_of::<T>(),
            );
            let buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(target, Some(buffer));
            gl.buffer_data_u8_slice(target, bytes, usage);

            buffer
        }
    }
}

impl<T> Default for GlBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}
