use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::math::transform::TransformOffset;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InoxNodeUuid(pub(crate) u32);

#[derive(Debug)]
pub struct InoxNode {
	pub uuid: InoxNodeUuid,
	pub name: String,
	pub type_name: String,
	pub enabled: bool,
	pub zsort: f32,
	pub trans_offset: TransformOffset,
	pub lock_to_root: bool,
	pub components: Components,
}

#[derive(Debug, Default)]
pub struct Components(HashMap<TypeId, Box<dyn Any>>);

impl Components {
	pub fn add<T: 'static>(&mut self, component: T) {
		self.0.insert(TypeId::of::<T>(), Box::new(component));
	}

	pub fn extend(&mut self, components: Components) {
		self.0.extend(components.0);
	}

	pub fn remove<T: 'static>(&mut self) -> Option<Box<T>> {
		self.0.remove(&TypeId::of::<T>()).and_then(|c| c.downcast().ok())
	}

	pub fn get<T: 'static>(&self) -> Option<&T> {
		self.0.get(&TypeId::of::<T>()).and_then(|c| c.downcast_ref::<T>())
	}

	pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
		self.0.get_mut(&TypeId::of::<T>()).and_then(|c| c.downcast_mut::<T>())
	}

	pub fn has<T: 'static>(&self) -> bool {
		self.0.contains_key(&TypeId::of::<T>())
	}
}
