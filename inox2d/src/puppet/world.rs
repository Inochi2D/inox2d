use std::any::TypeId;
use std::cell::UnsafeCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::hash::RandomState;
use std::mem::{size_of, transmute, ManuallyDrop, MaybeUninit};

use rayon::current_num_threads;
use rayon::prelude::*;

use super::InoxNodeUuid;

// to keep the provenance of the pointer in Vec (or any data struct that contains pointers),
// after transmutation they should be hosted in such a container for the compiler to properly reason
type VecBytes = [MaybeUninit<u8>; size_of::<Vec<()>>()];

/// type erased vec, only for World use. all methods unsafe as correctness solely dependent on usage
// so vec_bytes is aligned to the most
#[cfg_attr(target_pointer_width = "32", repr(align(4)))]
#[cfg_attr(target_pointer_width = "64", repr(align(8)))]
#[repr(C)]
struct AnyVec {
	vec_bytes: VecBytes,
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
		let vec = ManuallyDrop::new(Vec::<UnsafeCell<T>>::new());
		Self {
			// SAFETY: ManuallyDrop guaranteed to have same bit layout as inner, and inner is a proper Vec
			// provenance considerations present, see comment for VecBytes
			vec_bytes: unsafe { transmute::<ManuallyDrop<std::vec::Vec<UnsafeCell<T>>>, VecBytes>(vec) },
			// SAFETY: only to be called once at end of lifetime, and vec_bytes contain a valid Vec throughout self lifetime
			drop: |vec_bytes| unsafe {
				let vec: Vec<T> = transmute(*vec_bytes);
				// be explicit :)
				drop(vec);
			},
		}
	}

	/// T MUST be the same as in new::<T>() for a same instance
	pub unsafe fn downcast_unchecked<T>(&self) -> &Vec<UnsafeCell<T>> {
		transmute(&self.vec_bytes)
	}

	/// T MUST be the same as in new::<T>() for a same instance
	pub unsafe fn downcast_mut_unchecked<T>(&mut self) -> &mut Vec<UnsafeCell<T>> {
		transmute(&mut self.vec_bytes)
	}
}

pub struct World {
	// Type -> (Column, Ownership)
	columns: HashMap<TypeId, (AnyVec, HashMap<InoxNodeUuid, usize>)>,
}

pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

impl World {
	pub fn new() -> Self {
		Self {
			columns: HashMap::new(),
		}
	}

	/// adding a second component of the same type for a same node
	/// - panics in debug
	/// - silently discard the add in release
	pub fn add<T: Component>(&mut self, node: InoxNodeUuid, v: T) {
		let pair = self
			.columns
			.entry(TypeId::of::<T>())
			.or_insert((AnyVec::new::<T>(), HashMap::new()));
		// SAFETY: AnyVec in pair must be of type T, enforced by hashing
		let column = unsafe { pair.0.downcast_mut_unchecked() };

		debug_assert!(!pair.1.contains_key(&node),);
		pair.1.insert(node, column.len());
		column.push(UnsafeCell::new(v));
	}

	pub fn get<T: Component>(&self, node: InoxNodeUuid) -> Option<&T> {
		let pair = match self.columns.get(&TypeId::of::<T>()) {
			Some(c) => c,
			None => return None,
		};
		// SAFETY: AnyVec in pair must be of type T, enforced by hashing
		let column = unsafe { pair.0.downcast_unchecked() };

		let index = match pair.1.get(&node) {
			Some(i) => *i,
			None => return None,
		};
		debug_assert!(index < column.len());
		// SAFETY: what has been inserted into pair.1 should be a valid index
		Some(unsafe { &*column.get_unchecked(index).get() })
	}

	pub fn get_mut<T: Component>(&mut self, node: InoxNodeUuid) -> Option<&mut T> {
		let pair = match self.columns.get_mut(&TypeId::of::<T>()) {
			Some(c) => c,
			None => return None,
		};
		// SAFETY: AnyVec in pair must be of type T, enforced by hashing
		let column = unsafe { pair.0.downcast_mut_unchecked() };

		let index = match pair.1.get(&node) {
			Some(i) => *i,
			None => return None,
		};
		debug_assert!(index < column.len());
		// SAFETY: what has been inserted into pair.1 should be a valid index
		// SAFETY: since we own a &mut self, no other &mut references can exist
		// to any other world cell
		Some(unsafe { &mut *column.get_unchecked_mut(index).get() })
	}

	/// # Safety
	///
	/// All safety requirements of `UnsafeCell` apply. Caller must ensure no
	/// two aliasing mutable references to the same (node, T) pair exist at the
	/// same time.
	pub unsafe fn get_interior_mut<T: Component>(&self, node: InoxNodeUuid) -> Option<&mut T> {
		let pair = match self.columns.get(&TypeId::of::<T>()) {
			Some(c) => c,
			None => return None,
		};
		// SAFETY: AnyVec in pair must be of type T, enforced by hashing
		let column = unsafe { pair.0.downcast_unchecked() };

		let index = match pair.1.get(&node) {
			Some(i) => *i,
			None => return None,
		};
		debug_assert!(index < column.len());
		// SAFETY: what has been inserted into pair.1 should be a valid index
		Some(unsafe { &mut *column.get_unchecked(index).get() })
	}

	/// # Safety
	///
	/// call if node has added a comp of type T earlier
	pub unsafe fn get_unchecked<T: Component>(&self, node: InoxNodeUuid) -> &T {
		let pair = self.columns.get(&TypeId::of::<T>()).unwrap_unchecked();
		let index = *pair.1.get(&node).unwrap_unchecked();

		&*pair.0.downcast_unchecked().get_unchecked(index).get()
	}

	/// # Safety
	///
	/// call if node has added a comp of type T earlier
	pub unsafe fn get_mut_unchecked<T: Component>(&mut self, node: InoxNodeUuid) -> &mut T {
		let pair = self.columns.get_mut(&TypeId::of::<T>()).unwrap_unchecked();
		let index = *pair.1.get(&node).unwrap_unchecked();

		&mut *pair.0.downcast_mut_unchecked().get_unchecked_mut(index).get()
	}

	pub fn par_iter<F>(&mut self, nodes: &[InoxNodeUuid], your_fn: F)
	where
		F: for<'a> Fn(Partition<'a>) + Sync,
	{
		// SAFETY: We must filter duplicates from `nodes` to ensure disjoint
		// partitions.
		//
		// NOTE: For some reason type inference fails and we have to manually
		// tell Rust to use the standard state type.
		let nodes_set: HashSet<InoxNodeUuid, RandomState> = HashSet::from_iter(nodes.iter().map(|n| *n));
		let nodes_count = nodes_set.len();
		let batch_size = max(nodes_count / current_num_threads(), 1);

		let nodes_clean: Vec<_> = nodes_set.into_iter().collect();

		nodes_clean
			.par_chunks(batch_size)
			.for_each(|nodes| your_fn(unsafe { Partition::new_unchecked(nodes, &self) }))
	}
}

impl Default for World {
	fn default() -> Self {
		Self::new()
	}
}

/// A subset of the World that is partitioned by node UUID.
///
/// Partitions will allow access to the specific UUIDs only, with all other
/// access to nodes failing. This is intended as a way to partition
/// multithreaded writes to the ECS world by node.
pub struct Partition<'a> {
	nodes: &'a [InoxNodeUuid],
	data: &'a World,
}

impl<'a> Partition<'a> {
	/// Create a new world partition.
	///
	/// SAFETY: Callers must ensure that all live partitions of the given World
	/// reference disjoint node IDs before handing them to safe code.
	pub unsafe fn new_unchecked(nodes: &'a [InoxNodeUuid], data: &'a World) -> Self {
		Self { nodes, data }
	}

	pub fn get<T: Component>(&self, node: InoxNodeUuid) -> Option<&T> {
		// Note: the node scan is still necessary as some other partition may
		// be handing out &mut Ts
		if self.contains(node) {
			self.data.get(node)
		} else {
			None
		}
	}

	pub fn get_mut<T: Component>(&self, node: InoxNodeUuid) -> Option<&mut T> {
		if self.contains(node) {
			unsafe { self.data.get_interior_mut(node) }
		} else {
			None
		}
	}

	/// Retrieve the list of nodes present in the partition.
	pub fn nodes(&self) -> impl Iterator<Item = InoxNodeUuid> + use<'_> {
		self.nodes.iter().copied()
	}

	pub fn contains(&self, node: InoxNodeUuid) -> bool {
		self.nodes.iter().find(|n| **n == node).is_some()
	}
}

#[cfg(test)]
mod tests {
	mod any_vec {
		use super::super::AnyVec;
		use std::cell::UnsafeCell;

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
				any_vec
					.downcast_mut_unchecked()
					.push(UnsafeCell::new(Data { int: 0, c: b'A' }));
				any_vec
					.downcast_mut_unchecked()
					.push(UnsafeCell::new(Data { int: 1, c: b'B' }));
				any_vec
					.downcast_mut_unchecked()
					.push(UnsafeCell::new(Data { int: 2, c: b'C' }));

				assert_eq!(*any_vec.downcast_unchecked::<Data>()[0].get(), Data { int: 0, c: b'A' });
				assert_eq!(*any_vec.downcast_unchecked::<Data>()[1].get(), Data { int: 1, c: b'B' });

				(*any_vec.downcast_mut_unchecked::<Data>()[2].get()).c = b'D';

				assert_eq!(*any_vec.downcast_unchecked::<Data>()[2].get(), Data { int: 2, c: b'D' });
			}
		}
	}

	mod world {
		use super::super::{InoxNodeUuid, World};

		struct CompA {}
		struct CompB {
			i: u32,
		}
		struct CompC {
			f: f64,
		}

		const NODE_0: InoxNodeUuid = InoxNodeUuid(2);
		const NODE_1: InoxNodeUuid = InoxNodeUuid(3);
		const NODE_2: InoxNodeUuid = InoxNodeUuid(5);

		#[test]
		fn safe_ops() {
			let mut world = World::new();

			assert!(world.get::<CompA>(NODE_0).is_none());

			world.add(NODE_0, CompA {});
			world.add(NODE_0, CompB { i: 114 });
			world.add(NODE_0, CompC { f: 5.14 });
			world.add(NODE_1, CompA {});
			world.add(NODE_2, CompA {});
			world.add(NODE_1, CompC { f: 19.19 });
			world.add(NODE_2, CompC { f: 8.10 });

			assert!(world.get::<CompA>(NODE_0).is_some());
			assert!(world.get::<CompB>(NODE_1).is_none());

			assert_eq!(world.get::<CompB>(NODE_0).unwrap().i, 114);
			assert_eq!(world.get::<CompC>(NODE_2).unwrap().f, 8.10);

			world.get_mut::<CompC>(NODE_2).unwrap().f = 8.93;

			assert_eq!(world.get::<CompC>(NODE_2).unwrap().f, 8.93);
		}

		#[test]
		fn unsafe_ops() {
			let mut world = World::default();

			world.add(NODE_0, CompA {});
			world.add(NODE_0, CompB { i: 114 });
			world.add(NODE_0, CompC { f: 5.14 });
			world.add(NODE_1, CompA {});
			world.add(NODE_2, CompA {});
			world.add(NODE_1, CompC { f: 19.19 });
			world.add(NODE_2, CompC { f: 8.10 });

			unsafe {
				assert_eq!(world.get_unchecked::<CompB>(NODE_0).i, 114);
				assert_eq!(world.get_unchecked::<CompC>(NODE_2).f, 8.10);

				world.get_mut_unchecked::<CompC>(NODE_2).f = 8.93;

				assert_eq!(world.get_unchecked::<CompC>(NODE_2).f, 8.93);
			}
		}
	}
}
