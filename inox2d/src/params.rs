use glam::{vec2, Vec2};

use crate::math::{
	deform::Deform,
	interp::{bi_interpolate_f32, bi_interpolate_vec2s_additive, InterpRange, InterpolateMode},
	matrix::Matrix2d,
};
use crate::node::{
	components::{deform_stack::DeformSrc, DeformStack, TexturedMesh},
	InoxNodeUuid,
};
use crate::puppet::Puppet;
// use crate::render::{NodeRenderCtxs, PartRenderCtx, RenderCtxKind};

/// Parameter binding to a node. This allows to animate a node based on the value of the parameter that owns it.
pub struct Binding {
	pub node: InoxNodeUuid,
	pub is_set: Matrix2d<bool>,
	pub interpolate_mode: InterpolateMode,
	pub values: BindingValues,
}

#[derive(Debug, Clone)]
pub enum BindingValues {
	ZSort(Matrix2d<f32>),
	TransformTX(Matrix2d<f32>),
	TransformTY(Matrix2d<f32>),
	TransformSX(Matrix2d<f32>),
	TransformSY(Matrix2d<f32>),
	TransformRX(Matrix2d<f32>),
	TransformRY(Matrix2d<f32>),
	TransformRZ(Matrix2d<f32>),
	Deform(Matrix2d<Vec<Vec2>>),
}

#[derive(Debug, Clone)]
pub struct AxisPoints {
	pub x: Vec<f32>,
	pub y: Vec<f32>,
}

fn ranges_out(
	matrix: &Matrix2d<f32>,
	x_mindex: usize,
	x_maxdex: usize,
	y_mindex: usize,
	y_maxdex: usize,
) -> (InterpRange<f32>, InterpRange<f32>) {
	let out_top = InterpRange::new(matrix[(x_mindex, y_mindex)], matrix[(x_maxdex, y_mindex)]);
	let out_btm = InterpRange::new(matrix[(x_mindex, y_maxdex)], matrix[(x_maxdex, y_maxdex)]);
	(out_top, out_btm)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamUuid(pub u32);

/// Parameter. A simple bounded value that is used to animate nodes through bindings.
pub struct Param {
	pub uuid: ParamUuid,
	pub name: String,
	pub is_vec2: bool,
	pub min: Vec2,
	pub max: Vec2,
	pub defaults: Vec2,
	pub axis_points: AxisPoints,
	pub bindings: Vec<Binding>,
}

impl Param {
	/// Internal function that modifies puppet data according to one param set.
	/// Must be only called ONCE per frame to ensure correct behavior.
	///
	/// End users may repeatedly apply a same parameter for multiple times in between frames,
	/// but other facilities should be present to make sure this `apply()` is only called once per parameter.
	pub(crate) fn apply(&self, val: Vec2, puppet: &mut Puppet) {
		let val = val.clamp(self.min, self.max);
		let val_normed = (val - self.min) / (self.max - self.min);

		// calculate axis point indexes
		let (x_mindex, x_maxdex) = {
			let x_temp = self.axis_points.x.binary_search_by(|a| a.total_cmp(&val_normed.x));

			match x_temp {
				Ok(ind) if ind == self.axis_points.x.len() - 1 => (ind - 1, ind),
				Ok(ind) => (ind, ind + 1),
				Err(ind) => (ind - 1, ind),
			}
		};

		let (y_mindex, y_maxdex) = {
			let y_temp = self.axis_points.y.binary_search_by(|a| a.total_cmp(&val_normed.y));

			match y_temp {
				Ok(ind) if ind == self.axis_points.y.len() - 1 => (ind - 1, ind),
				Ok(ind) => (ind, ind + 1),
				Err(ind) => (ind - 1, ind),
			}
		};

		// Apply offset on each binding
		for binding in &self.bindings {
			let node_offsets = puppet.nodes.get_node_mut(binding.node).unwrap();

			let range_in = InterpRange::new(
				vec2(self.axis_points.x[x_mindex], self.axis_points.y[y_mindex]),
				vec2(self.axis_points.x[x_maxdex], self.axis_points.y[y_maxdex]),
			);

			match binding.values {
				BindingValues::ZSort(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.zsort +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformTX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.translation.x +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformTY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.translation.y +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformSX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.scale.x *=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformSY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.scale.y *=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformRX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.rotation.x +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformRY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.rotation.y +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformRZ(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					node_offsets.trans_offset.rotation.z +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::Deform(ref matrix) => {
					let out_top = InterpRange::new(
						matrix[(x_mindex, y_mindex)].as_slice(),
						matrix[(x_maxdex, y_mindex)].as_slice(),
					);
					let out_bottom = InterpRange::new(
						matrix[(x_mindex, y_maxdex)].as_slice(),
						matrix[(x_maxdex, y_maxdex)].as_slice(),
					);

					// deform specified by a parameter must be direct, i.e., in the form of displacements of all vertices
					let direct_deform = {
						let textured_mesh = puppet.node_comps.get::<TexturedMesh>(binding.node);
						if let Some(textured_mesh) = textured_mesh {
							let vert_len = textured_mesh.mesh.vertices.len();
							let mut direct_deform: Vec<Vec2> = Vec::with_capacity(vert_len);
							direct_deform.resize(vert_len, Vec2::ZERO);

							bi_interpolate_vec2s_additive(
								val_normed,
								range_in,
								out_top,
								out_bottom,
								binding.interpolate_mode,
								&mut direct_deform,
							);

							direct_deform
						} else {
							todo!("Deform on node types other than Part.")
						}
					};

					puppet
						.node_comps
						.get_mut::<DeformStack>(binding.node)
						.expect("Nodes being deformed must have a DeformStack component.")
						.push(DeformSrc::Param(self.uuid), Deform::Direct(direct_deform));
				}
			}
		}
	}
}

/*
impl Puppet {
	pub fn get_param(&self, uuid: ParamUuid) -> Option<&Param> {
		self.params.get(&uuid)
	}

	pub fn get_param_mut(&mut self, uuid: ParamUuid) -> Option<&mut Param> {
		self.params.get_mut(&uuid)
	}

	pub fn get_named_param(&self, name: &str) -> Option<&Param> {
		self.params.get(self.param_names.get(name)?)
	}

	pub fn get_named_param_mut(&mut self, name: &str) -> Option<&mut Param> {
		self.params.get_mut(self.param_names.get(name)?)
	}

	pub fn begin_set_params(&mut self) {
		// Reset all transform and deform offsets before applying bindings
		for (key, value) in self.render_ctx.node_render_ctxs.iter_mut() {
			value.trans_offset = self.nodes.get_node(*key).expect("node to be in tree").trans_offset;
		}

		for v in self.render_ctx.vertex_buffers.deforms.iter_mut() {
			*v = Vec2::ZERO;
		}
	}

	pub fn end_set_params(&mut self, dt: f32) {
		// TODO: find better places for these two update calls and pass elapsed time in
		self.update_physics(dt, self.physics);
		self.update_trans();
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SetParamError {
	#[error("No parameter named {0}")]
	NoParameterNamed(String),

	#[error("No parameter with uuid {0:?}")]
	NoParameterWithUuid(ParamUuid),
}

impl Puppet {
	pub fn set_named_param(&mut self, param_name: &str, val: Vec2) -> Result<(), SetParamError> {
		let Some(param_uuid) = self.param_names.get(param_name) else {
			return Err(SetParamError::NoParameterNamed(param_name.to_string()));
		};

		self.set_param(*param_uuid, val)
	}

	pub fn set_param(&mut self, param_uuid: ParamUuid, val: Vec2) -> Result<(), SetParamError> {
		let Some(param) = self.params.get_mut(&param_uuid) else {
			return Err(SetParamError::NoParameterWithUuid(param_uuid));
		};

		param.apply(
			val,
			&mut self.render_ctx.node_render_ctxs,
			self.render_ctx.vertex_buffers.deforms.as_mut_slice(),
		);

		Ok(())
	}
}
*/
