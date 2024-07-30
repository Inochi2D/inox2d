use std::ops::Deref;

/// Component holding zsort values that may be modified across frames.
// only one value instead of absolute + relative as in TransformStore, cause inheritance of zsort (+) is commutative
#[derive(Default)]
pub(crate) struct ZSort(pub f32);

// so ZSort automatically gets the `.total_cmp()` of `f32`
impl Deref for ZSort {
	type Target = f32;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
