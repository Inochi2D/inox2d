pub mod pendulum;
mod runge_kutta;
mod simple_physics;

use crate::nodes::node_data::InoxData;
use crate::params::ParamUuid;
use crate::puppet::Puppet;
use pendulum::Pendulum;

use glam::Vec2;

/// Physics model to use for simple physics
#[derive(Debug, Clone)]
pub enum SimplePhysicsSystem {
    /// Rigid pendulum
    Pendulum(Pendulum),
    // TODO: Springy pendulum
    // SpringPendulum(),
}

impl SimplePhysicsSystem {
    pub fn tick(&mut self, anchor: &Vec2, props: &SimplePhysicsProps, dt: f32) -> Vec2 {
        // enum dispatch, fill the branches once other systems are implemented
        // as for inox2d, users are not expected to bring their own physics system,
        // no need to do dynamic dispatch with something like Box<dyn SimplePhysicsSystem>
        match self {
            SimplePhysicsSystem::Pendulum(system) => system.tick(anchor, props, dt),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SimplePhysicsProps {
    /// Gravity scale (1.0 = puppet gravity)
    pub gravity: f32,
    /// Pendulum/spring rest length (pixels)
    pub length: f32,
    /// Resonant frequency (Hz)
    pub frequency: f32,
    /// Angular damping ratio
    pub angle_damping: f32,
    /// Length damping ratio
    pub length_damping: f32,

    pub output_scale: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParamMapMode {
    AngleLength,
    XY,
}

#[derive(Debug, Clone)]
pub struct SimplePhysics {
    pub param: ParamUuid,

    pub system: SimplePhysicsSystem,
    pub map_mode: ParamMapMode,

    pub offset_props: SimplePhysicsProps,
    pub props: SimplePhysicsProps,

    /// Whether physics system listens to local transform only.
    pub local_only: bool,

    pub anchor: Vec2,
    pub output: Vec2,
}

impl Puppet {
    /// Update the puppet's nodes' absolute transforms, by applying further displacements yielded by the physics system
    /// in response to displacements caused by parameter changes
    pub fn update_physics(&mut self, dt: f32) {
        for driver_uuid in self.drivers.clone() {
            let Some(driver) = self.nodes.get_node_mut(driver_uuid) else {
                continue;
            };
            let InoxData::SimplePhysics(ref mut system) = driver.data else {
                continue;
            };
            let nrc = &self.render_ctx.node_render_ctxs[&driver.uuid];

            let output = system.update(dt, nrc, system.param);
            let param_uuid = system.param;
            self.set_param(param_uuid, output);
        }
    }
}
