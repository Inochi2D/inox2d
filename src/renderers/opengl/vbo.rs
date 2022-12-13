use glow::HasContext;

pub enum Vbo<T: Copy> {
    Buffering(Vec<T>),
    Uploaded(glow::NativeBuffer),
}

impl<T: Copy> Vbo<T> {
    pub fn new() -> Vbo<T> {
        Vbo::Buffering(Vec::new())
    }

    pub fn from(vec: Vec<T>) -> Vbo<T> {
        Vbo::Buffering(vec)
    }

    pub fn len(&self) -> usize {
        match self {
            Vbo::Buffering(vec) => vec.len(),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn extend_from_slice(&mut self, other: &[T]) {
        match self {
            Vbo::Buffering(vec) => vec.extend_from_slice(other),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, other: I) {
        match self {
            Vbo::Buffering(vec) => vec.extend(other),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub fn upload(&mut self, gl: &glow::Context, target: u32, usage: u32) {
        match self {
            Vbo::Buffering(vec) => {
                let slice = &vec;
                unsafe {
                    let bytes: &[u8] = core::slice::from_raw_parts(
                        slice.as_ptr() as *const u8,
                        slice.len() * core::mem::size_of::<T>(),
                    );
                    let vbo = gl.create_buffer().unwrap();
                    gl.bind_buffer(target, Some(vbo));
                    gl.buffer_data_u8_slice(target, bytes, usage);
                    *self = Vbo::Uploaded(vbo);
                }
            }
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }
}

impl<T: Copy> Default for Vbo<T> {
    fn default() -> Self {
        Self::new()
    }
}
