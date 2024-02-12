use glam::{vec2, Vec2};

use crate::physics::runge_kutta::{self, PhysicsState};
use crate::physics::SimplePhysicsProps;
use crate::puppet::PuppetPhysics;

/// Marker type for a rigid pendulum physics state
struct RigidPendulum;

type RigidPendulumState = PhysicsState<2, RigidPendulum>;

#[allow(unused)]
impl RigidPendulumState {
    // θ

    pub fn get_vθ(&self) -> f32 {
        self.vars[0]
    }

    pub fn set_vθ(&mut self, value: f32) {
        self.vars[0] = value;
    }

    pub fn get_dθ(&self) -> f32 {
        self.derivatives[0]
    }

    pub fn set_dθ(&mut self, value: f32) {
        self.derivatives[0] = value;
    }

    // ω

    pub fn get_vω(&self) -> f32 {
        self.vars[1]
    }

    pub fn set_vω(&mut self, value: f32) {
        self.vars[1] = value;
    }

    pub fn get_dω(&self) -> f32 {
        self.derivatives[1]
    }

    pub fn set_dω(&mut self, value: f32) {
        self.derivatives[1] = value;
    }
}

fn eval(
    state: &mut RigidPendulumState,
    &(puppet_physics, props): &(PuppetPhysics, &SimplePhysicsProps),
    _anchor: Vec2,
    _t: f32,
) {
    // https://www.myphysicslab.com/pendulum/pendulum-en.html

    let g = props.gravity * puppet_physics.pixels_per_meter * puppet_physics.gravity;
    let r = props.length;

    // θ' = ω
    state.set_dθ(state.get_vω());

    // ω' = -(g/R) sin θ
    let dω = -(g / r) * state.get_vθ().sin();

    // critical damp: that way a damping value of 1 corresponds to no bouncing
    let crit_damp = 2. * (g / r).sqrt();

    let damping = -state.get_vω() * props.angle_damping * crit_damp;

    state.set_dω(dω + damping);
}

#[derive(Debug, Clone, Default)]
pub struct RigidPendulumSystem {
    pub bob: Vec2,
    state: RigidPendulumState,
}

impl RigidPendulumSystem {
    pub fn tick(
        &mut self,
        anchor: Vec2,
        puppet_physics: PuppetPhysics,
        props: &SimplePhysicsProps,
        dt: f32,
    ) -> Vec2 {
        // Compute the angle against the updated anchor position
        let d_bob = self.bob - anchor;
        self.state.set_vθ(f32::atan2(-d_bob.x, d_bob.y));

        // Run the pendulum simulation in terms of angle
        runge_kutta::tick(&eval, &mut self.state, (puppet_physics, props), anchor, dt);

        // Update the bob position at the new angle
        let angle = self.state.get_vθ();
        let d_bob = vec2(-angle.sin(), angle.cos());
        self.bob = anchor + d_bob * props.length;

        self.bob
    }
}
