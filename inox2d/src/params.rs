use std::collections::HashMap;

use glam::{vec2, Vec2};

use crate::math::{
	deform::Deform,
	interp::{bi_interpolate_f32, bi_interpolate_vec2s_additive, InterpRange, InterpolateMode},
	matrix::Matrix2d,
};
use crate::node::{
	components::{DeformSource, DeformStack, Mesh, TransformStore, ZSort},
	InoxNodeUuid,
};
use crate::puppet::{Puppet, World};

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
	/// Internal function that modifies puppet components according to one param set.
	/// Must be only called ONCE per frame to ensure correct behavior.
	///
	/// End users may repeatedly apply a same parameter for multiple times in between frames,
	/// but other facilities should be present to make sure this `apply()` is only called once per parameter.
	pub(crate) fn apply(&self, val: Vec2, comps: &mut World) {
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
			let range_in = InterpRange::new(
				vec2(self.axis_points.x[x_mindex], self.axis_points.y[y_mindex]),
				vec2(self.axis_points.x[x_maxdex], self.axis_points.y[y_maxdex]),
			);

			match binding.values {
				BindingValues::ZSort(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps.get_mut::<ZSort>(binding.node).unwrap().0 +=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformTX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps
						.get_mut::<TransformStore>(binding.node)
						.unwrap()
						.relative
						.translation
						.x += bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformTY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps
						.get_mut::<TransformStore>(binding.node)
						.unwrap()
						.relative
						.translation
						.y += bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformSX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps.get_mut::<TransformStore>(binding.node).unwrap().relative.scale.x *=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformSY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps.get_mut::<TransformStore>(binding.node).unwrap().relative.scale.y *=
						bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformRX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps
						.get_mut::<TransformStore>(binding.node)
						.unwrap()
						.relative
						.rotation
						.x += bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformRY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps
						.get_mut::<TransformStore>(binding.node)
						.unwrap()
						.relative
						.rotation
						.y += bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
				}
				BindingValues::TransformRZ(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					comps
						.get_mut::<TransformStore>(binding.node)
						.unwrap()
						.relative
						.rotation
						.z += bi_interpolate_f32(val_normed, range_in, out_top, out_bottom, binding.interpolate_mode);
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
						let mesh = comps
							.get::<Mesh>(binding.node)
							.expect("Deform param target must have an associated Mesh.");

						let vert_len = mesh.vertices.len();
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
					};

					comps
						.get_mut::<DeformStack>(binding.node)
						.expect("Nodes being deformed must have a DeformStack component.")
						.push(DeformSource::Param(self.uuid), Deform::Direct(direct_deform));
				}
			}
		}
	}
}

/// Additional struct attached to a puppet for animating through params.
pub struct ParamCtx {
	values: HashMap<String, Vec2>,
}

impl ParamCtx {
	pub(crate) fn new(puppet: &Puppet) -> Self {
		Self {
			values: puppet.params.iter().map(|p| (p.0.to_owned(), p.1.defaults)).collect(),
		}
	}

	/// Reset all params to default value.
	pub(crate) fn reset(&mut self, params: &HashMap<String, Param>) {
		for (name, value) in self.values.iter_mut() {
			*value = params.get(name).unwrap().defaults;
		}
	}

	/// Set param with name to value `val`.
	pub fn set(&mut self, param_name: &str, val: Vec2) -> Result<(), SetParamError> {
		if let Some(value) = self.values.get_mut(param_name) {
			*value = val;
			Ok(())
		} else {
			Err(SetParamError::NoParameterNamed(param_name.to_string()))
		}
	}

	/// Modify components as specified by all params. Must be called ONCE per frame.
	pub(crate) fn apply(&self, params: &HashMap<String, Param>, comps: &mut World) {
		// a correct implementation should not care about the order of `.apply()`
		for (param_name, val) in self.values.iter() {
			// TODO: a correct implementation should not fail on param value (0, 0)
			if *val != Vec2::ZERO {
				params.get(param_name).unwrap().apply(*val, comps);
			}
		}
	}
}

/// Possible errors setting a param.
#[derive(Debug, thiserror::Error)]
pub enum SetParamError {
	#[error("No parameter named {0}")]
	NoParameterNamed(String),
}
