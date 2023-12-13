use std::f32::consts::PI;
use std::ops::Not;

use glam::{vec2, vec4, Vec2};

use crate::puppet::{Puppet, PuppetPhysics};
use crate::render::NodeRenderCtx;
use crate::system::{ParamMapMode, PhysicsSystem, SimplePhysicsSystem};

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

#[derive(Debug, Clone)]
pub struct SimplePhysics {
    pub param: u32,

    pub system: SimplePhysicsSystem,
    pub map_mode: ParamMapMode,

    pub offset_props: SimplePhysicsProps,
    pub props: SimplePhysicsProps,

    /// Whether physics system listens to local transform only.
    pub local_only: bool,

    pub anchor: Vec2,
    pub output: Vec2,
}

impl SimplePhysics {
    pub fn tick_system(&mut self, h: f32) {
        match &mut self.system {
            SimplePhysicsSystem::Pendulum(system) => tick(system, &self.props, h),
            // spring pendulum when
        }
    }

    fn update_inputs(&mut self, node_render_ctx: &NodeRenderCtx) {
        let anchor_pos = match self.local_only {
            true => node_render_ctx.trans_offset.translation.extend(1.0),
            false => node_render_ctx.trans * vec4(0.0, 0.0, 0.0, 1.0),
        };

        self.anchor = vec2(anchor_pos.x, anchor_pos.y);
    }

    fn update_outputs(
        &mut self,
        node_render_ctx: &NodeRenderCtx,
        puppet: &mut Puppet,
        param_name: &str,
    ) {
        let oscale = self.final_output_scale();

        // "Okay, so this is confusing. We want to translate the angle back to local space, but not the coordinates."
        // - Asahi Lina

        // Transform the physics output back into local space.
        // The origin here is the anchor. This gives us the local angle.
        let local_pos4 = match self.local_only {
            true => vec4(self.output.x, self.output.y, 0.0, 1.0),
            false => node_render_ctx.trans.inverse() * vec4(self.output.x, self.output.y, 0.0, 1.0),
        };

        let local_angle = vec2(local_pos4.x, local_pos4.y).normalize();

        // Figure out the relative length. We can work this out directly in global space.
        let relative_length = self.output.distance(self.anchor) / self.final_length();

        let param_value = match self.map_mode {
            ParamMapMode::XY => {
                let local_pos_norm = local_angle * relative_length;
                let mut result = local_pos_norm - Vec2::Y;
                result.y = -result.y; // Y goes up for params
                result
            }
            ParamMapMode::AngleLength => {
                let a = f32::atan2(-local_angle.x, local_angle.y) / PI;
                vec2(a, relative_length)
            }
        };

        puppet.set_param(param_name, param_value * oscale);
    }

    pub fn update_driver(
        &mut self,
        dt: f32,
        node_render_ctx: &NodeRenderCtx,
        puppet: &mut Puppet,
        param_name: &str,
    ) {
        // Timestep is limited to 10 seconds.
        // If you're getting 0.1 FPS, you have bigger issues to deal with.
        let mut h = dt.min(10.);

        self.update_inputs(node_render_ctx);

        // Minimum physics timestep: 0.01s
        while h > 0.01 {
            self.tick_system(0.01);
            h -= 0.01;
        }

        self.tick_system(h);

        self.update_outputs(node_render_ctx, puppet, param_name);
    }

    pub fn update_anchor(&mut self) {
        let bob = self.anchor + vec2(0.0, self.final_length());

        match &mut self.system {
            SimplePhysicsSystem::Pendulum(system) => system.bob = bob,
            // spring pendulum when
        }
    }

    pub fn final_gravity(&self, puppet_physics: &PuppetPhysics) -> f32 {
        (self.props.gravity * self.offset_props.gravity)
            * puppet_physics.gravity
            * puppet_physics.pixels_per_meter
    }

    pub fn final_length(&self) -> f32 {
        self.props.length * self.offset_props.length
    }

    pub fn final_frequency(&self) -> f32 {
        self.props.frequency * self.offset_props.frequency
    }

    pub fn final_angle_damping(&self) -> f32 {
        self.props.angle_damping * self.offset_props.angle_damping
    }

    pub fn final_length_damping(&self) -> f32 {
        self.props.length_damping * self.offset_props.length_damping
    }

    pub fn final_output_scale(&self) -> Vec2 {
        self.props.output_scale * self.offset_props.output_scale
    }
}

pub fn tick<const N: usize, P: PhysicsSystem<N>>(
    system: &mut P,
    physics_props: &SimplePhysicsProps,
    h: f32,
) {
    let curs;
    let t = {
        let phys = system.state_mut();
        curs = phys.vars;
        phys.derivatives = [0.; N];
        phys.t
    };

    let phys = system.eval(physics_props, t);
    let k1s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * k1s[i] / 2.;
    }
    let phys = system.eval(physics_props, t + h / 2.);
    let k2s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * k2s[i] / 2.;
    }
    let phys = system.eval(physics_props, t + h / 2.);
    let k3s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * k3s[i];
    }
    let phys = system.eval(physics_props, t + h);
    let k4s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * (k1s[i] + 2. * k2s[i] + 2. * k3s[i] + k4s[i]) / 6.;
        if phys.vars[i].is_finite().not() {
            // Simulation failed, revert
            phys.vars = curs;
            break;
        }
    }

    phys.t += h;
}
