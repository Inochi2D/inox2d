use std::collections::HashMap;
use std::mem::swap;

use glam::Vec2;

use crate::math::deform::{linear_combine, Deform};
use crate::node::components::{DeformSource, DeformStack};
use crate::puppet::{InoxNodeTree, World};

impl DeformStack {
	pub(crate) fn new(deform_len: usize) -> Self {
		Self {
			deform_len,
			stack: HashMap::new(),
		}
	}

	/// Reset the stack. Ready to receive deformations for one frame.
	pub(crate) fn reset(&mut self) {
		for enabled_deform in self.stack.values_mut() {
			enabled_deform.0 = false;
		}
	}

	/// Combine the deformations received so far according to some rules, and write to the result
	pub(crate) fn combine(&self, _nodes: &InoxNodeTree, _node_comps: &World, result: &mut [Vec2]) {
		if result.len() != self.deform_len {
			panic!("Required output deform dimensions different from what DeformStack is initialized with.")
		}

		let direct_deforms = self.stack.values().filter_map(|enabled_deform| {
			if enabled_deform.0 {
				let Deform::Direct(ref direct_deform) = enabled_deform.1;
				Some(direct_deform)
			} else {
				None
			}
		});
		linear_combine(direct_deforms, result);
	}

	/// Submit a deform from a source for a node.
	pub(crate) fn push(&mut self, src: DeformSource, mut deform: Deform) {
		let Deform::Direct(ref direct_deform) = deform;
		if direct_deform.len() != self.deform_len {
			panic!("A direct deform with non-matching dimensions is submitted to a node.");
		}

		self.stack
			.entry(src)
			.and_modify(|enabled_deform| {
				if enabled_deform.0 {
					panic!("A same source submitted deform twice for a same node within one frame.")
				}
				enabled_deform.0 = true;

				swap(&mut enabled_deform.1, &mut deform);
			})
			.or_insert((true, deform));
	}
}
