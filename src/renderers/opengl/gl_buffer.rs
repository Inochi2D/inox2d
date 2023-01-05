// Copyright (c) 2022 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(target, Some(vbo));
            gl.buffer_data_u8_slice(target, bytes, usage);

            vbo
        }
    }
}

impl<T> Default for GlBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}
