use glow::HasContext;

pub(super) enum Vbo<T: Copy> {
    Buffering(Vec<T>),
    Uploaded(glow::NativeBuffer),
}

impl<T: Copy> Vbo<T> {
    pub(super) fn new() -> Vbo<T> {
        Vbo::Buffering(Vec::new())
    }

    pub(super) fn from(vec: Vec<T>) -> Vbo<T> {
        Vbo::Buffering(vec)
    }

    pub(super) fn len(&self) -> usize {
        match self {
            Vbo::Buffering(vec) => vec.len(),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub(super) fn extend_from_slice(&mut self, other: &[T]) {
        match self {
            Vbo::Buffering(vec) => vec.extend_from_slice(other),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub(super) fn extend<I: IntoIterator<Item = T>>(&mut self, other: I) {
        match self {
            Vbo::Buffering(vec) => vec.extend(other),
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }

    pub(super) unsafe fn upload(&self, gl: &glow::Context, slot: u32) {
        match self {
            Vbo::Buffering(vec) => {
                let slice = &vec;
                let bytes: &[u8] = core::slice::from_raw_parts(
                    slice.as_ptr() as *const u8,
                    slice.len() * core::mem::size_of::<T>(),
                );
                gl.buffer_data_u8_slice(slot, bytes, glow::STATIC_DRAW);
            }
            _ => panic!("Vbo must not be uploaded yet!"),
        }
    }
}
