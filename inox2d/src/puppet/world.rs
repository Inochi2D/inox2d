use std::any::TypeId;
use std::collections::HashMap;
use std::mem::{size_of, transmute, ManuallyDrop, MaybeUninit};

// to keep the provenance of the pointer in Vec (or any data struct that contains pointers),
// after transmutation they should be hosted in such a container for the compiler to properly reason
type VecBytes = [MaybeUninit<u8>; size_of::<Vec<()>>()];

/// type erased vec
// so vec_bytes is aligned to the most
#[cfg_attr(target_pointer_width = "32", repr(align(4)))]
#[cfg_attr(target_pointer_width = "64", repr(align(8)))]
#[repr(C)]
struct AnyVec {
	vec_bytes: VecBytes,
	type_id: TypeId,
	drop: fn(&mut VecBytes),
}

impl Drop for AnyVec {
	fn drop(&mut self) {
		(self.drop)(&mut self.vec_bytes);
	}
}

impl AnyVec {
	// Self is inherently Send + Sync as a pack of bytes regardless of inner type, which is bad
	pub fn new<T: 'static + Send + Sync>() -> Self {
		let vec = ManuallyDrop::new(Vec::<T>::new());
		Self {
			// SAFETY: ManuallyDrop guaranteed to have same bit layout as inner, and inner is a proper Vec
			// provenance considerations present, see comment for VecBytes
			vec_bytes: unsafe { transmute(vec) },
			type_id: TypeId::of::<T>(),
			// SAFETY: only to be called once at end of lifetime, and vec_bytes contain a valid Vec throughout self lifetime
			drop: |vec_bytes| unsafe {
				let vec: Vec<T> = transmute(*vec_bytes);
				// be explicit :)
				drop(vec);
			},
		}
	}

	/// T MUST be the same as in new::<T>() for a same instance
	pub unsafe fn downcast_unchecked<T>(&self) -> &Vec<T> {
		transmute(&self.vec_bytes)
	}

	/// T MUST be the same as in new::<T>() for a same instance
	pub unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut Vec<T> {
		transmute(&mut self.vec_bytes)
	}

	pub fn downcast<T: 'static>(&self) -> Option<&Vec<T>> {
		if TypeId::of::<T>() == self.type_id {
			// SAFETY: T is the same as in new::<T>()
			Some(unsafe { self.downcast_unchecked() })
		} else {
			None
		}
	}

	pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut Vec<T>> {
		if TypeId::of::<T>() == self.type_id {
			// SAFETY: T is the same as in new::<T>()
			Some(unsafe { self.downcast_mut_unchecked() })
		} else {
			None
		}
	}
}

pub struct World {
	columns: HashMap<TypeId, AnyVec>,
}

#[cfg(test)]
mod tests {
	mod any_vec {
		use super::super::AnyVec;

		#[test]
		fn new_and_drop_empty() {
			let any_vec = AnyVec::new::<[u8; 1]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 2]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 3]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 4]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 5]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 6]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 7]>();
			drop(any_vec);
			let any_vec = AnyVec::new::<[u8; 8]>();
			drop(any_vec);
		}

		#[test]
		fn push_and_get_and_set() {
			#[derive(Debug, PartialEq, Eq)]
			struct Data {
				int: u32,
				c: u8,
			}

			let mut any_vec = AnyVec::new::<Data>();

			unsafe {
				any_vec.downcast_mut_unchecked().push(Data { int: 0, c: b'A' });
				any_vec.downcast_mut_unchecked().push(Data { int: 1, c: b'B' });
				any_vec.downcast_mut_unchecked().push(Data { int: 2, c: b'C' });

				assert_eq!(any_vec.downcast_unchecked::<Data>()[0], Data { int: 0, c: b'A' });
				assert_eq!(any_vec.downcast_unchecked::<Data>()[1], Data { int: 1, c: b'B' });

				any_vec.downcast_mut_unchecked::<Data>()[2].c = b'D';

				assert_eq!(any_vec.downcast_unchecked::<Data>()[2], Data { int: 2, c: b'D' });
			}
		}

		#[test]
		fn safety() {
			#[derive(Debug, PartialEq, Eq)]
			struct Data {
				int: u32,
			}
			struct OtherData {}

			let mut any_vec = AnyVec::new::<Data>();

			assert!(any_vec.downcast::<Data>().is_some());
			assert!(any_vec.downcast_mut::<Data>().is_some());

			any_vec.downcast_mut::<Data>().unwrap().push(Data { int: 1 });
			assert_eq!(any_vec.downcast::<Data>().unwrap()[0], Data { int: 1 });

			assert!(any_vec.downcast::<OtherData>().is_none());
			assert!(any_vec.downcast_mut::<OtherData>().is_none());
		}
	}
}
