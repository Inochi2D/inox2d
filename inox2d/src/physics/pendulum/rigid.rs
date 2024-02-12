use glam::{vec2, Vec2};

use crate::physics::runge_kutta::{self, PhysicsState};
use crate::physics::SimplePhysicsProps;

/// Marker type for a rigid pendulum physics state
struct RigidPendulum;

type RigidPendulumState = PhysicsState<2, RigidPendulum>;

#[allow(unused)]
impl RigidPendulumState {
    // θ

    pub fn vθ(&self) -> f32 {
        self.vars[0]
    }

    pub fn vθ_mut(&mut self) -> &mut f32 {
        &mut self.vars[0]
    }

    pub fn dθ(&self) -> f32 {
        self.derivatives[0]
    }

    pub fn dθ_mut(&mut self) -> &mut f32 {
        &mut self.derivatives[0]
    }

    // ω

    pub fn vω(&self) -> f32 {
        self.vars[1]
    }

    pub fn vω_mut(&mut self) -> &mut f32 {
        &mut self.vars[1]
    }

    pub fn dω(&self) -> f32 {
        self.derivatives[1]
    }

    pub fn dω_mut(&mut self) -> &mut f32 {
        &mut self.derivatives[1]
    }
}

fn eval(state: &mut RigidPendulumState, props: &SimplePhysicsProps, _anchor: Vec2, _t: f32) {
    // https://www.myphysicslab.com/pendulum/pendulum-en.html

    let g = props.gravity;
    let r = props.length;

    // θ' = ω
    *state.dθ_mut() = state.vω();

    // ω' = -(g/R) sin θ
    let dω = -(g / r) * state.vθ().sin();

    // critical damp: that way a damping value of 1 corresponds to no bouncing
    let crit_damp = 2. * (g / r).sqrt();

    let damping = -state.vω() * props.angle_damping * crit_damp;

    *state.dω_mut() = dω + damping;
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
        *self.state.vθ_mut() = f32::atan2(-d_bob.x, d_bob.y);

        // Run the pendulum simulation in terms of angle
        runge_kutta::tick(&eval, &mut self.state, props, anchor, dt);

        // Update the bob position at the new angle
        let angle = self.state.vθ();
        let d_bob = vec2(-angle.sin(), angle.cos());
        self.bob = anchor + d_bob * props.length;

        self.bob
    }
}
