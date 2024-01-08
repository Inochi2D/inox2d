use glam::{Vec2, vec2};

use crate::physics::runge_kutta::{PhysicsState, self};
use crate::physics::SimplePhysicsProps;

/// Marker type for a rigid pendulum physics state
struct RigidPendulum;

type RigidPendulumState = PhysicsState<2, RigidPendulum>;

impl RigidPendulumState {
    // angle

    pub fn getv_angle(&self) -> f32 {
        self.vars[0]
    }

    pub fn setv_angle(&mut self, angle: f32) {
        self.vars[0] = angle;
    }

    // pub fn getd_angle(&self) -> f32 {
    //     self.derivatives[0]
    // }

    pub fn setd_angle(&mut self, angle: f32) {
        self.derivatives[0] = angle;
    }

    // dangle

    pub fn getv_dangle(&self) -> f32 {
        self.vars[1]
    }

    // pub fn setv_dangle(&mut self, dangle: f32) {
    //     self.vars[1] = dangle;
    // }

    // pub fn getd_dangle(&self) -> f32 {
    //     self.derivatives[1]
    // }

    pub fn setd_dangle(&mut self, dangle: f32) {
        self.derivatives[1] = dangle;
    }
}

fn eval(
    state: &mut RigidPendulumState,
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

#[derive(Debug, Clone, Default)]
pub struct RigidPendulumSystem {
    pub bob: Vec2,
    state: RigidPendulumState,
}

impl RigidPendulumSystem {
    pub fn tick(&mut self, anchor: Vec2, props: &SimplePhysicsProps, dt: f32) -> Vec2 {
        // Compute the angle against the updated anchor position
        let d_bob = self.bob - anchor;
        self.state.setv_angle(f32::atan2(-d_bob.x, d_bob.y));

        // Run the pendulum simulation in terms of angle
        runge_kutta::tick(&eval, &mut self.state, props, anchor, dt);

        // Update the bob position at the new angle
        let angle = self.state.getv_angle();
        let d_bob = vec2(-angle.sin(), angle.cos());
        self.bob = anchor + d_bob * props.length;

        self.bob
    }
}
