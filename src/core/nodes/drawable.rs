use std::sync::atomic::{AtomicU32, Ordering, AtomicBool};

static DRAWABLE_VAO: AtomicU32 = AtomicU32::new(0);

pub(crate) static DO_GENERATE_BOUNDS: AtomicBool = AtomicBool::new(false);

pub(crate) fn in_init_drawable() {
    #[cfg(feature = "in_does_render")]
    {
        let mut drawable_vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut drawable_vao) }
        DRAWABLE_VAO.store(drawable_vao, Ordering::Release);
    }
}

/// Binds the internal vertex array for rendering.
pub(crate) fn in_drawable_bind_vao() {
    // Bind our vertex array
    unsafe { gl::BindVertexArray(DRAWABLE_VAO.load(Ordering::Relaxed)) };
}

pub fn in_set_update_bounds(state: bool) {
    DO_GENERATE_BOUNDS.store(state, Ordering::Release);
}
