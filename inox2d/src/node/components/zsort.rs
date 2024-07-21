use std::ops::Deref;

#[derive(Default)]
pub(crate) struct ZSort(pub f32);

// so ZSort automatically gets the `.total_cmp()` of `f32`
impl Deref for ZSort {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
