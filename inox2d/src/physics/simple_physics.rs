use std::f32::consts::PI;

use glam::{vec2, vec4, Vec2};

use super::{ParamMapMode, SimplePhysics, SimplePhysicsSystem};
use crate::puppet::PuppetPhysics;
use crate::render::NodeRenderCtx;

impl SimplePhysics {
    fn update_inputs(&mut self, node_render_ctx: &NodeRenderCtx) {
        let anchor_pos = match self.local_only {
            true => node_render_ctx.trans_offset.translation.extend(1.0),
            false => node_render_ctx.trans * vec4(0.0, 0.0, 0.0, 1.0),
        };

        self.anchor = vec2(anchor_pos.x, anchor_pos.y);
    }

    fn calc_outputs(&self, node_render_ctx: &NodeRenderCtx) -> Vec2 {
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

        param_value * oscale
    }

    pub fn update(&mut self, dt: f32, node_render_ctx: &NodeRenderCtx) -> Vec2 {
        // Timestep is limited to 10 seconds.
        // If you're getting 0.1 FPS, you have bigger issues to deal with.
        let mut h = dt.min(10.);

        self.update_inputs(node_render_ctx);

        // Minimum physics timestep: 0.01s
        while h > 0.01 {
            self.tick(0.01);
            h -= 0.01;
        }

        self.tick(h);

        self.calc_outputs(node_render_ctx)
    }

    pub fn update_anchor(&mut self) {
        let new_bob = self.anchor + vec2(0.0, self.final_length());

        match &mut self.system {
            SimplePhysicsSystem::RigidPendulum { ref mut bob, .. } => *bob = new_bob,
            SimplePhysicsSystem::SpringPendulum { ref mut bob, .. } => *bob = new_bob,
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
