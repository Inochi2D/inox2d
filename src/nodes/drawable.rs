use std::sync::atomic::{AtomicU32, Ordering};

static DRAWABLE_VAO: AtomicU32 = AtomicU32::new(0);

pub(crate) fn in_init_drawable() {
    #[cfg(feature = "in_does_render")]
    {
        let mut drawable_vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut drawable_vao) }
        DRAWABLE_VAO.store(drawable_vao, Ordering::Release);
    }
}
