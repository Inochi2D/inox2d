pub(crate) trait IsPhysicsVars<const N: usize> {
	fn get_f32s(&self) -> [f32; N];
	fn set_f32s(&mut self, f32s: [f32; N]);
}

#[derive(Default)]
pub(crate) struct PhysicsState<const N: usize, T: Default + IsPhysicsVars<N>> {
	pub vars: T,
	pub derivatives: T,
}

impl<const N: usize, T: Default + IsPhysicsVars<N>> PhysicsState<N, T> {
	pub fn tick<P, A>(
		&mut self,
		eval: &impl Fn(&mut PhysicsState<N, T>, &P, &A, f32),
		props: P,
		anchor: &A,
		t: f32,
		h: f32,
	) {
		let curs = self.vars.get_f32s();
		self.derivatives.set_f32s([0.; N]);

		(eval)(self, &props, anchor, t);
		let k1s = self.derivatives.get_f32s();

		let mut vars = [0.; N];
		for i in 0..N {
			vars[i] = curs[i] + h * k1s[i] / 2.;
		}
		self.vars.set_f32s(vars);
		(eval)(self, &props, anchor, t + h / 2.);
		let k2s = self.derivatives.get_f32s();

		let mut vars = [0.; N];
		for i in 0..N {
			vars[i] = curs[i] + h * k2s[i] / 2.;
		}
		self.vars.set_f32s(vars);
		(eval)(self, &props, anchor, t + h / 2.);
		let k3s = self.derivatives.get_f32s();

		let mut vars = [0.; N];
		for i in 0..N {
			vars[i] = curs[i] + h * k3s[i];
		}
		self.vars.set_f32s(vars);
		(eval)(self, &props, anchor, t + h);
		let k4s = self.derivatives.get_f32s();

		let mut vars = [0.; N];
		for i in 0..N {
			vars[i] = curs[i] + h * (k1s[i] + 2. * k2s[i] + 2. * k3s[i] + k4s[i]) / 6.;
			if !vars[i].is_finite() {
				// Simulation failed, revert
				vars = curs;
				break;
			}
		}
		self.vars.set_f32s(vars);
	}
}
