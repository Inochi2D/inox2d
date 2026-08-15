use glam::Mat4;

use crate::{
	math::{deform::Deform, triangle::MeshBitMask},
	node::components::{DeformSource, DeformStack, Mesh, MeshGroup, TransformStore, ZSort},
};

use super::{InoxNodeTree, Puppet, World};

pub(crate) struct TransformCtx {}

impl TransformCtx {
	/// Give every node a `TransformStore` and a `ZSort` component, if the puppet is going to be rendered/animated
	pub fn new(puppet: &mut Puppet) -> Self {
		for node in puppet.nodes.iter() {
			puppet.node_comps.add(node.uuid, TransformStore::default());
			puppet.node_comps.add(node.uuid, ZSort::default());
		}
		TransformCtx {}
	}

	/// Reset all transform/zsort values to default.
	pub fn reset(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		for node in nodes.iter() {
			comps.get_mut::<TransformStore>(node.uuid).unwrap().relative = node.trans_offset.clone();
			comps.get_mut::<ZSort>(node.uuid).unwrap().0 = node.zsort;
		}
	}

	/// Update the puppet's nodes' absolute transforms, by combining transforms
	/// from each node's ancestors in a pre-order traversal manner.
	pub(crate) fn update(&mut self, nodes: &InoxNodeTree, comps: &mut World) {
		let root_trans_store = comps.get_mut::<TransformStore>(nodes.root_node_id).unwrap();
		// The root's absolute transform is its relative transform.
		let root_trans = root_trans_store.relative.to_matrix();
		root_trans_store.absolute = root_trans;

		let root_zsort = comps.get_mut::<ZSort>(nodes.root_node_id).unwrap().0;

		// Pre-order traversal, just the order to ensure that parents are accessed earlier than children
		// Skip the root
		for node in nodes.pre_order_iter().skip(1) {
			let base = if node.lock_to_root {
				(root_trans, root_zsort)
			} else {
				let parent = nodes.get_parent(node.uuid);
				(
					comps.get_mut::<TransformStore>(parent.uuid).unwrap().absolute,
					comps.get_mut::<ZSort>(parent.uuid).unwrap().0,
				)
			};

			let node_trans_store = comps.get_mut::<TransformStore>(node.uuid).unwrap();
			let node_trans = node_trans_store.relative.to_matrix();
			node_trans_store.absolute = base.0 * node_trans;

			let node_zsort = comps.get_mut::<ZSort>(node.uuid).unwrap();
			node_zsort.0 += base.1;

			// Since this is the only place I can think of to put it, let's go
			// and look for the closest MeshGroup parent and add that to the
			// stack.

			let mut ancestor = nodes.get_parent(node.uuid).uuid;
			let mut tf_matrix = if let Some(transform_store) = comps.get::<TransformStore>(node.uuid) {
				transform_store.relative.to_matrix()
			} else {
				Mat4::IDENTITY
			};

			while ancestor != nodes.root_node_id {
				let imposes_deform = if let Some(mg) = comps.get::<MeshGroup>(ancestor) {
					if mg.dynamic {
						true
					} else {
						// Static meshgroups do not need a deform stack item.
						break;
					}
				} else {
					// Meshes can impose a deform on their children anyway
					comps.get::<Mesh>(ancestor).is_some()
				};

				if imposes_deform {
					// We need to calculate a testing mask for all our parent
					// deform meshes.
					if comps.get::<MeshBitMask>(ancestor).is_none() {
						comps.add(ancestor, MeshBitMask::new(comps.get::<Mesh>(ancestor).unwrap()));
					}

					if let Some(deform_stack) = comps.get_mut::<DeformStack>(node.uuid) {
						deform_stack.push(DeformSource::MeshGroup(ancestor), Deform::Source(tf_matrix));
						break;
					}
				}

				if let Some(transform_store) = comps.get::<TransformStore>(ancestor) {
					tf_matrix = transform_store.relative.to_matrix() * tf_matrix;
				}

				ancestor = nodes.get_parent(ancestor).uuid;
			}
		}
	}
}
