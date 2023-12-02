use std::ops::Not;

use glam::Vec2;

use crate::system::{PhysicsSystem, SimplePhysicsSystem, ParamMapMode};

#[derive(Debug, Clone)]
pub struct SimplePhysics {
    pub param: u32,

    pub offset_gravity: f32,
    pub offset_length: f32,
    pub offset_frequency: f32,
    pub offset_angle_damping: f32,
    pub offset_length_damping: f32,
    pub offset_output_scale: Vec2,

    pub system: SimplePhysicsSystem,
    pub map_mode: ParamMapMode,

    /// Whether physics system listens to local transform only.
    pub local_only: bool,
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
    pub anchor: Vec2,
    pub output: Vec2,
}

impl SimplePhysics {
    pub fn tick<const N: usize, P: PhysicsSystem<N>>(&self, system: &mut P, h: f32) {
        let curs;
        let t = {
            let phys = system.state_mut();
            curs = phys.vars;
            phys.derivatives = [0.; N];
            phys.t
        };

        let phys = system.eval(self, t);
        let k1s = phys.derivatives;

        for i in 0..N {
            phys.vars[i] = curs[i] + h * k1s[i] / 2.;
        }
        let phys = system.eval(self, t + h / 2.);
        let k2s = phys.derivatives;

        for i in 0..N {
            phys.vars[i] = curs[i] + h * k2s[i] / 2.;
        }
        let phys = system.eval(self, t + h / 2.);
        let k3s = phys.derivatives;

        for i in 0..N {
            phys.vars[i] = curs[i] + h * k3s[i];
        }
        let phys = system.eval(self, t + h);
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

    fn update_inputs(&mut self) {
        // let anchor_pos = match self.local_only {
        //     true => ...,
        //     false => ...,
        // }
    }

    fn update_outputs(&mut self) {
        // TODO
    }

    pub fn update_driver(&mut self, dt: f32) {
        // Timestep is limited to 10 seconds.
        // If you're getting 0.1 FPS, you have bigger issues to deal with.
        let h = dt.min(10.);


    }
}
