use std::collections::HashMap;
use std::convert::identity;

use glam::{vec2, Vec2};

use crate::math::interp::{bi_interpolate_f32, bi_interpolate_vec2s_additive, InterpRange, InterpolateMode};
use crate::math::matrix::Matrix2d;
use crate::math::transform::Transform;
use crate::nodes::node::InoxNodeUuid;

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

/// Parameter. A simple bounded value that is used to animate nodes through bindings.
#[derive(Debug, Clone)]
pub struct Param {
    pub uuid: u32,
    pub name: String,
    pub is_vec2: bool,
    pub min: Vec2,
    pub max: Vec2,
    pub defaults: Vec2,
    pub axis_points: AxisPoints,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone, Default)]
pub struct PartOffsets {
    pub vert_offset: u16,
    pub vert_len: usize,
    pub trans_offset: Transform,
}

impl Param {
    pub fn apply<Po: AsMut<PartOffsets>>(
        &self,
        val: Vec2,
        node_offsets: &mut HashMap<InoxNodeUuid, Po>,
        deform_buf: &mut [Vec2],
    ) {
        let val = val.clamp(self.min, self.max);
        let val_normed = (val - self.min) / (self.max - self.min);

        // calculate axis point indexes
        let x_mindex = self
            .axis_points
            .x
            .binary_search_by(|x| x.total_cmp(&val_normed.x))
            .map_or_else(identity, identity)
            .clamp(0, self.axis_points.x.len().saturating_sub(2));
        let x_maxdex = x_mindex + 1;

        let y_mindex = self
            .axis_points
            .y
            .binary_search_by(|y| y.total_cmp(&val_normed.y))
            .map_or_else(identity, identity)
            .clamp(0, self.axis_points.y.len().saturating_sub(2));
        let y_maxdex = y_mindex + 1;

        // Apply offset on each binding
        for binding in &self.bindings {
            let part_offsets = node_offsets.get_mut(&binding.node).unwrap();
            let part_offsets = part_offsets.as_mut();

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
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.translation.x += bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
                }
                BindingValues::TransformTY(ref matrix) => {
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.translation.y += bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
                }
                BindingValues::TransformSX(ref matrix) => {
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.scale.x *= bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
                }
                BindingValues::TransformSY(ref matrix) => {
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.scale.y *= bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
                }
                BindingValues::TransformRX(ref matrix) => {
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.rotation.x += bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
                }
                BindingValues::TransformRY(ref matrix) => {
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.rotation.y += bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
                }
                BindingValues::TransformRZ(ref matrix) => {
                    let out_top = InterpRange::new(
                        matrix[(x_mindex, y_mindex)],
                        matrix[(x_maxdex, y_mindex)],
                    );
                    let out_bottom = InterpRange::new(
                        matrix[(x_mindex, y_maxdex)],
                        matrix[(x_maxdex, y_maxdex)],
                    );

                    part_offsets.trans_offset.rotation.z += bi_interpolate_f32(
                        val_normed,
                        range_in,
                        out_top,
                        out_bottom,
                        binding.interpolate_mode,
                    );
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

                    let def_beg = part_offsets.vert_offset as usize;
                    let def_end = def_beg + part_offsets.vert_len;

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
