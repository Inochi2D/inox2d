use crate::physics::runge_kutta::PhysicsState;
use crate::physics::SimplePhysicsProps;
use glam::{vec2, Vec2};
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct SpringPendulum;

impl PhysicsState<4, SpringPendulum> {
    // bob

    pub fn getv_bob(&self) -> Vec2 {
        vec2(self.vars[0], self.vars[1])
    }

    pub fn setv_bob(&mut self, bob: Vec2) {
        self.vars[0] = bob.x;
        self.vars[1] = bob.y;
    }

    pub fn getd_bob(&self) -> Vec2 {
        vec2(self.derivatives[0], self.derivatives[1])
    }

    pub fn setd_bob(&mut self, bob: Vec2) {
        self.derivatives[0] = bob.x;
        self.derivatives[1] = bob.y;
    }

    // dbob

    pub fn getv_dbob(&self) -> Vec2 {
        vec2(self.vars[2], self.vars[3])
    }

    pub fn setv_dbob(&mut self, dbob: Vec2) {
        self.vars[2] = dbob.x;
        self.vars[3] = dbob.y;
    }

    pub fn getd_dbob(&self) -> Vec2 {
        vec2(self.derivatives[2], self.derivatives[3])
    }

    pub fn setd_dbob(&mut self, dbob: Vec2) {
        self.derivatives[2] = dbob.x;
        self.derivatives[3] = dbob.y;
    }
}

pub fn eval(
    state: &mut PhysicsState<4, SpringPendulum>,
    props: &SimplePhysicsProps,
    anchor: Vec2,
    _t: f32,
) {
    state.setd_bob(state.getv_dbob());

    // These are normalized vs. mass
    let spring_ksqrt = props.frequency * 2. * PI;
    let spring_k = spring_ksqrt.powi(2);

    let g = props.gravity;
    let rest_length = props.length - g / spring_k;

    let off_pos = state.getv_bob() - anchor;
    let off_pos_norm = off_pos.normalize();

    let length_ratio = props.gravity / props.length;
    let crit_damp_angle = 2. * length_ratio.sqrt();
    let crit_damp_length = 2. * spring_ksqrt;

    let dist = anchor.distance(state.getv_bob()).abs();
    let force = vec2(0., g) - (off_pos_norm * (dist - rest_length) * spring_k);

    let d_bob = state.getv_dbob();
    let d_bob_rot = vec2(
        d_bob.x * off_pos_norm.y + d_bob.y * off_pos_norm.x,
        d_bob.y * off_pos_norm.y - d_bob.x * off_pos_norm.x,
    );

    let dd_bob_rot = -vec2(
        d_bob_rot.x * props.angle_damping * crit_damp_angle,
        d_bob_rot.y * props.length_damping * crit_damp_length,
    );

    let dd_bob_damping = vec2(
        dd_bob_rot.x * off_pos_norm.y - d_bob_rot.y * off_pos_norm.x,
        dd_bob_rot.y * off_pos_norm.y + d_bob_rot.x * off_pos_norm.x,
    );

    let dd_bob = force + dd_bob_damping;

    state.setd_dbob(dd_bob);
}
