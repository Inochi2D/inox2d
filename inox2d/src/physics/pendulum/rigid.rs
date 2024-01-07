use glam::Vec2;

use crate::physics::runge_kutta::PhysicsState;
use crate::physics::SimplePhysicsProps;

pub struct RigidPendulum;

impl PhysicsState<2, RigidPendulum> {
    // angle

    pub fn getv_angle(&self) -> f32 {
        self.vars[0]
    }

    pub fn setv_angle(&mut self, angle: f32) {
        self.vars[0] = angle;
    }

    pub fn getd_angle(&self) -> f32 {
        self.derivatives[0]
    }

    pub fn setd_angle(&mut self, angle: f32) {
        self.derivatives[0] = angle;
    }

    // dangle

    pub fn getv_dangle(&self) -> f32 {
        self.vars[1]
    }

    pub fn setv_dangle(&mut self, dangle: f32) {
        self.vars[1] = dangle;
    }

    pub fn getd_dangle(&self) -> f32 {
        self.derivatives[1]
    }

    pub fn setd_dangle(&mut self, dangle: f32) {
        self.derivatives[1] = dangle;
    }
}

pub fn eval(
    state: &mut PhysicsState<2, RigidPendulum>,
    physics_props: &SimplePhysicsProps,
    _anchor: Vec2,
    _t: f32,
) {
    state.setd_angle(state.getv_dangle());

    let dd = {
        let length_ratio = physics_props.gravity / physics_props.length;
        let crit_damp = 2. * length_ratio.sqrt();
        let dd = -length_ratio * state.getv_angle().sin();
        dd - state.getv_dangle() * physics_props.angle_damping * crit_damp
    };

    state.setd_dangle(dd);
}
