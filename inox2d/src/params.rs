use std::collections::HashMap;

use glam::{vec2, Vec2};

use crate::math::{
	deform::Deform,
	interp::{bi_interpolate_f32, bi_interpolate_vec2s_additive, InterpRange, InterpolateMode},
	matrix::Matrix2d,
};
use crate::node::{
	components::{DeformSource, DeformStack, Drawable, Mesh, TransformStore, ZSort},
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
	Opacity(Matrix2d<f32>),
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
		let range = self.max - self.min;
		let val_normed = vec2(
			if range.x > 0.0 {
				(val.x - self.min.x) / range.x
			} else {
				0.0
			},
			if range.y > 0.0 {
				(val.y - self.min.y) / range.y
			} else {
				0.0
			},
		);

		// calculate axis point indexes
		let (x_mindex, x_maxdex) = Self::find_indices(&self.axis_points.x, val_normed.x);
		let (y_mindex, y_maxdex) = Self::find_indices(&self.axis_points.y, val_normed.y);

		let range_in = InterpRange::new(
			vec2(
				self.axis_points.x.get(x_mindex).copied().unwrap_or(0.0),
				self.axis_points.y.get(y_mindex).copied().unwrap_or(0.0),
			),
			vec2(
				self.axis_points.x.get(x_maxdex).copied().unwrap_or(0.0),
				self.axis_points.y.get(y_maxdex).copied().unwrap_or(0.0),
			),
		);

		// Clamp normalized value to the selected range to avoid interpolation artifacts
		let val_clamped = val_normed.clamp(range_in.beg, range_in.end);

		// Apply each binding
		for binding in &self.bindings {
			match binding.values {
				BindingValues::ZSort(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(zsort) = comps.get_mut::<ZSort>(binding.node) {
						zsort.0 +=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformTX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.translation.x +=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformTY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.translation.y +=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformSX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.scale.x *=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformSY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.scale.y *=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformRX(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.rotation.x +=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformRY(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.rotation.y +=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::TransformRZ(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(trans) = comps.get_mut::<TransformStore>(binding.node) {
						trans.relative.rotation.z +=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
				BindingValues::Deform(ref matrix) => {
					let out_top = InterpRange::new(
						matrix.get(x_mindex, y_mindex).map(|v| v.as_slice()).unwrap_or(&[]),
						matrix.get(x_maxdex, y_mindex).map(|v| v.as_slice()).unwrap_or(&[]),
					);
					let out_bottom = InterpRange::new(
						matrix.get(x_mindex, y_maxdex).map(|v| v.as_slice()).unwrap_or(&[]),
						matrix.get(x_maxdex, y_maxdex).map(|v| v.as_slice()).unwrap_or(&[]),
					);

					let mesh = comps.get::<Mesh>(binding.node).unwrap_or_else(|| {
						panic!(
							"Deform param target must have an associated Mesh. (Binding Node ID: {:?})",
							binding.node.0
						)
					});

					let vert_len = mesh.vertices.len();
					let mut direct_deform: Vec<Vec2> = vec![Vec2::ZERO; vert_len];

					bi_interpolate_vec2s_additive(
						val_clamped,
						range_in,
						out_top,
						out_bottom,
						binding.interpolate_mode,
						&mut direct_deform,
					);

					if let Some(deform_stack) = comps.get_mut::<DeformStack>(binding.node) {
						deform_stack.push(DeformSource::Param(self.uuid), Deform::Direct(direct_deform));
					}
				}
				BindingValues::Opacity(ref matrix) => {
					let (out_top, out_bottom) = ranges_out(matrix, x_mindex, x_maxdex, y_mindex, y_maxdex);

					if let Some(drawable) = comps.get_mut::<Drawable>(binding.node) {
						drawable.blending.opacity *=
							bi_interpolate_f32(val_clamped, range_in, out_top, out_bottom, binding.interpolate_mode);
					}
				}
			}
		}
	}

	fn find_indices(points: &[f32], t: f32) -> (usize, usize) {
		let len = points.len();
		if len == 0 {
			return (0, 0);
		}
		if len == 1 {
			return (0, 0);
		}

		match points.binary_search_by(|a| a.total_cmp(&t)) {
			Ok(ind) => {
				if ind + 1 < len {
					(ind, ind + 1)
				} else {
					(ind - 1, ind)
				}
			}
			Err(ind) => {
				if ind == 0 {
					(0, 1)
				} else if ind >= len {
					(len - 2, len - 1)
				} else {
					(ind - 1, ind)
				}
			}
		}
	}
}

/// Additional struct attached to a puppet for animating through params.
pub struct ParamCtx {
	pub(crate) values: HashMap<String, Vec2>,
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
			if let Some(param) = params.get(name) {
				*value = param.defaults;
			}
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
		for (param_name, val) in self.values.iter() {
			if let Some(param) = params.get(param_name) {
				// Apply even if it's (0, 0) to ensure correctness, as per TODO.
				// We can optimize this later if needed by checking against param.defaults.
				param.apply(*val, comps);
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
