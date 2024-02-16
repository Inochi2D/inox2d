use glam::Vec2;

pub trait IsPhysicsVars<const N: usize> {
	fn get_f32s(&self) -> [f32; N];
	fn set_f32s(&mut self, f32s: [f32; N]);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PhysicsState<T> {
	pub vars: T,
	pub derivatives: T,
	pub t: f32,
}

pub fn tick<const N: usize, T: IsPhysicsVars<N>, P>(
	eval: &impl Fn(&mut PhysicsState<T>, &P, Vec2, f32),
	phys: &mut PhysicsState<T>,
	props: P,
	anchor: Vec2,
	h: f32,
) {
	let curs = phys.vars.get_f32s();
	phys.derivatives.set_f32s([0.; N]);

	let t = phys.t;

	(eval)(phys, &props, anchor, t);
	let k1s = phys.derivatives.get_f32s();

	let mut vars = [0.; N];
	for i in 0..N {
		vars[i] = curs[i] + h * k1s[i] / 2.;
	}
	phys.vars.set_f32s(vars);
	(eval)(phys, &props, anchor, t + h / 2.);
	let k2s = phys.derivatives.get_f32s();

	let mut vars = [0.; N];
	for i in 0..N {
		vars[i] = curs[i] + h * k2s[i] / 2.;
	}
	phys.vars.set_f32s(vars);
	(eval)(phys, &props, anchor, t + h / 2.);
	let k3s = phys.derivatives.get_f32s();

	let mut vars = [0.; N];
	for i in 0..N {
		vars[i] = curs[i] + h * k3s[i];
	}
	phys.vars.set_f32s(vars);
	(eval)(phys, &props, anchor, t + h);
	let k4s = phys.derivatives.get_f32s();

	let mut vars = [0.; N];
	for i in 0..N {
		vars[i] = curs[i] + h * (k1s[i] + 2. * k2s[i] + 2. * k3s[i] + k4s[i]) / 6.;
		if !vars[i].is_finite() {
			// Simulation failed, revert
			phys.vars.set_f32s(curs);
			break;
		}
	}
	phys.vars.set_f32s(vars);

	phys.t += h;
}
