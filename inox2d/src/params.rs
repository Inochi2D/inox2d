use glam::{vec2, Vec2};

use crate::math::interp::{bi_interpolate_f32, bi_interpolate_vec2s_additive, InterpRange, InterpolateMode};
use crate::math::matrix::Matrix2d;
use crate::nodes::node::{InoxNodeUuid, InoxParamUuid};
use crate::puppet::Puppet;
use crate::render::{NodeRenderCtxs, PartRenderCtx, RenderCtxKind};

/// Parameter binding to a node. This allows to animate a node based on the value of the parameter that owns it.
#[derive(Debug, Clone)]
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

/// Parameter. A simple bounded value that is used to animate nodes through bindings.
#[derive(Debug, Clone)]
pub struct Param {
	pub uuid: InoxParamUuid,
	pub name: Option<String>,
	pub is_vec2: bool,
	pub min: Vec2,
	pub max: Vec2,
	pub defaults: Vec2,
	pub axis_points: AxisPoints,
	pub bindings: Vec<Binding>,
}

impl Param {
	pub fn apply(&self, val: Vec2, node_render_ctxs: &mut NodeRenderCtxs, deform_buf: &mut [Vec2]) {
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
			let node_offsets = node_render_ctxs.get_mut(&binding.node).unwrap();

			let range_in = InterpRange::new(
				vec2(self.axis_points.x[x_mindex], self.axis_points.y[y_mindex]),
				vec2(self.axis_points.x[x_maxdex], self.axis_points.y[y_maxdex]),
			);

			match binding.values {
				BindingValues::ZSort(_) => {
					// Seems complicated to do currently...
					// Do nothing for now
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

					if let RenderCtxKind::Part(PartRenderCtx {
						vert_offset, vert_len, ..
					}) = node_offsets.kind
					{
						let def_beg = vert_offset as usize;
						let def_end = def_beg + vert_len;

						bi_interpolate_vec2s_additive(
							val_normed,
							range_in,
							out_top,
							out_bottom,
							binding.interpolate_mode,
							&mut deform_buf[def_beg..def_end],
						);
					}
				}
			}
		}
	}
}

impl Puppet {
	pub fn get_param(&self, name: &str) -> Option<&Param> {
		self.param_names.get(name).map(|pu| {
			self.parameters
				.get(pu)
				.unwrap_or_else(|| panic!("param index mismatch {pu:?}"))
		})
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

	pub fn set_param(&mut self, param_name: &str, val: Vec2) {
		let param = self
			.param_names
			.get(param_name)
			.map(|pn| self.parameters.get(pn).expect("param index mismatch {pn}"))
			.unwrap_or_else(|| panic!("No parameter named: {}", param_name));

		param.apply(
			val,
			&mut self.render_ctx.node_render_ctxs,
			self.render_ctx.vertex_buffers.deforms.as_mut_slice(),
		);
	}

	pub fn end_set_params(&mut self) {
		self.update_trans();
	}
}
