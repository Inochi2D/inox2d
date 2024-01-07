use std::fmt;
use std::marker::PhantomData;

use glam::Vec2;

use super::SimplePhysicsProps;

pub struct PhysicsState<const N: usize, T> {
    pub vars: [f32; N],
    pub derivatives: [f32; N],
    pub t: f32,
    pub _data: PhantomData<T>,
}

impl<const N: usize, T> fmt::Debug for PhysicsState<N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicsState")
            .field("vars", &self.vars)
            .field("derivatives", &self.derivatives)
            .field("t", &self.t)
            .finish()
    }
}

impl<const N: usize, T> Clone for PhysicsState<N, T> {
    fn clone(&self) -> Self {
        Self {
            vars: self.vars,
            derivatives: self.derivatives,
            t: self.t,
            _data: self._data,
        }
    }
}

impl<const N: usize, T> Default for PhysicsState<N, T> {
    fn default() -> Self {
        Self {
            vars: [0.; N],
            derivatives: [0.; N],
            t: 0.,
            _data: PhantomData,
        }
    }
}

pub fn tick<const N: usize, T>(
	eval: &impl Fn(&mut PhysicsState<N, T>, &SimplePhysicsProps, Vec2, f32),
	phys: &mut PhysicsState<N, T>,
	physics_props: &SimplePhysicsProps,
    anchor: Vec2,
	h: f32,
) {
    let curs = phys.vars;
    phys.derivatives = [0.; N];

    let t = phys.t;

    (eval)(phys, physics_props, anchor, t);
    let k1s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * k1s[i] / 2.;
    }
    (eval)(phys, physics_props, anchor, t + h / 2.);
    let k2s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * k2s[i] / 2.;
    }
    (eval)(phys, physics_props, anchor, t + h / 2.);
    let k3s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * k3s[i];
    }
    (eval)(phys, physics_props, anchor, t + h);
    let k4s = phys.derivatives;

    for i in 0..N {
        phys.vars[i] = curs[i] + h * (k1s[i] + 2. * k2s[i] + 2. * k3s[i] + k4s[i]) / 6.;
        if !phys.vars[i].is_finite() {
            // Simulation failed, revert
            phys.vars = curs;
            break;
        }
    }

    phys.t += h;
}
