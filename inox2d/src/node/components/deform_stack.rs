use core::mem::swap;
use std::collections::HashMap;

use glam::Vec2;

use crate::math::deform::{linear_combine, Deform};
use crate::node::InoxNodeUuid;
use crate::params::ParamUuid;
use crate::puppet::{InoxNodeTree, World};

/// Source of a deform.
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub(crate) enum DeformSrc {
	Param(ParamUuid),
	Node(InoxNodeUuid),
}

/// Storing deforms specified by multiple sources to apply on one node for one frame.
///
/// Despite the name (this is respecting the ref impl), this is not in any way a stack.
/// The order of deforms being applied, or more generally speaking, the way multiple deforms adds up to be a single one, needs to be defined according to the spec.
pub(crate) struct DeformStack {
	/// this is a component so cannot use generics for the length.
	deform_len: usize,
	/// map of (src, (enabled, Deform)).
	/// On reset, only set enabled to false instead of clearing the map, as deforms from same sources tend to come in every frame.
	stack: HashMap<DeformSrc, (bool, Deform)>,
}

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
	pub(crate) fn push(&mut self, src: DeformSrc, mut deform: Deform) {
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
