use super::SimplePhysicsProps;

use std::ops::Not;

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

/// implement this trait to be able to use provided Runge-Kutta method implementation
pub trait PhysicsSystem<const N: usize> {
    fn state(&self) -> &PhysicsState<N>;
    fn state_mut(&mut self) -> &mut PhysicsState<N>;
    fn set_state(&mut self, state: PhysicsState<N>);

    fn eval(&mut self, physics_props: &SimplePhysicsProps, t: f32) -> &mut PhysicsState<N>;
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
