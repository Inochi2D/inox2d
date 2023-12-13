use glam::Vec2;

use crate::nodes::physics::{self, SimplePhysics, SimplePhysicsProps};

#[derive(Clone, Debug)]
pub struct PhysicsState<const N: usize> {
    pub vars: [f32; N],
    pub derivatives: [f32; N],
    pub t: f32,
}

impl<const N: usize> Default for PhysicsState<N> {
    fn default() -> Self {
        Self {
            vars: [0.; N],
            derivatives: [0.; N],
            t: 0.,
        }
    }
}

pub trait PhysicsSystem<const N: usize> {
    type Output;

    fn state(&self) -> &PhysicsState<N>;
    fn state_mut(&mut self) -> &mut PhysicsState<N>;
    fn set_state(&mut self, state: PhysicsState<N>);

    fn eval(&mut self, physics_props: &SimplePhysicsProps, t: f32) -> &mut PhysicsState<N>;
    fn tick(&mut self, node: &SimplePhysics, h: f32) -> Self::Output;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParamMapMode {
    AngleLength,
    XY,
}

#[derive(Default, Debug, Clone)]
pub struct Pendulum {
    /// bob is happy
    pub bob: Vec2,
    /// contains the angle and delta-angle
    physics_state: PhysicsState<2>,
}

impl Pendulum {
    pub fn angle(&self) -> f32 {
        self.physics_state.vars[0]
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.physics_state.vars[0] = angle;
    }

    pub fn set_derivative_angle(&mut self, angle: f32) {
        self.physics_state.derivatives[0] = angle;
    }

    pub fn delta_angle(&self) -> f32 {
        self.physics_state.vars[1]
    }

    pub fn set_delta_angle(&mut self, delta_angle: f32) {
        self.physics_state.vars[1] = delta_angle;
    }

    pub fn set_derivative_delta_angle(&mut self, delta_angle: f32) {
        self.physics_state.derivatives[1] = delta_angle;
    }
}

impl PhysicsSystem<2> for Pendulum {
    type Output = Vec2;

    fn state(&self) -> &PhysicsState<2> {
        &self.physics_state
    }

    fn state_mut(&mut self) -> &mut PhysicsState<2> {
        &mut self.physics_state
    }

    fn set_state(&mut self, state: PhysicsState<2>) {
        self.physics_state = state;
    }

    fn eval(&mut self, physics_props: &SimplePhysicsProps, _t: f32) -> &mut PhysicsState<2> {
        self.set_derivative_angle(self.delta_angle());

        let dd = {
            let length_ratio = physics_props.gravity / physics_props.length;
            let crit_damp = 2. * length_ratio.sqrt();
            let dd = -length_ratio * self.angle().sin();
            dd - self.delta_angle() * physics_props.angle_damping * crit_damp
        };
        self.set_derivative_delta_angle(dd);

        &mut self.physics_state
    }

    // TODO: oh my god there are 2 tick functions. This really needs a rename/refactor/whatever.

    fn tick(&mut self, node: &SimplePhysics, h: f32) -> Vec2 {
        // Compute the angle against the updated anchor position
        let delta_bob = self.bob - node.anchor;
        self.set_angle(f32::atan2(-delta_bob.x, delta_bob.y));

        // Run the pendulum simulation in terms of angle
        physics::tick(self, &node.props, h);

        // Update bob's position at the new angle
        let angle = self.angle();
        let delta_bob = Vec2::new(-angle.sin(), angle.cos());
        self.bob = node.anchor + delta_bob * node.props.length;

        self.bob
    }
}

/// Physics model to use for simple physics
#[derive(Debug, Clone)]
pub enum SimplePhysicsSystem {
    /// Rigid pendulum
    Pendulum(Pendulum),
    // TODO: Springy pendulum
    // SpringPendulum(),
}
