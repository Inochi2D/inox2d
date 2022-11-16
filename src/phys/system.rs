use glam::Mat4;

#[derive(Clone, Debug, Default)]
pub struct PhysicsState {
    pub vars: Vec<f32>,
    pub derivative: Vec<f32>,
    pub t: f32,
}

pub trait PhysicsSystem {
    fn eval(&mut self, t: f32) -> &mut PhysicsState;

    fn state(&self) -> &PhysicsState;
    fn state_mut(&mut self) -> &mut PhysicsState;
    fn set_state(&mut self, state: PhysicsState);

    fn tick(&mut self, h: f32) {
        let curs;
        let t = {
            let phys = self.state_mut();
            curs = phys.vars.clone();
            phys.derivative = vec![0.; curs.len()];
            phys.t
        };

        let phys = self.eval(t);
        let k1s = phys.derivative.clone();

        for ((var, cur), k) in phys.vars.iter_mut().zip(&curs).zip(&k1s) {
            *var = cur + h * k / 2.;
        }
        let phys = self.eval(t + h / 2.);
        let k2s = phys.derivative.clone();

        for ((var, cur), k) in phys.vars.iter_mut().zip(&curs).zip(&k2s) {
            *var = cur + h * k / 2.;
        }
        let phys = self.eval(t + h / 2.);
        let k3s = phys.derivative.clone();

        for ((var, cur), k) in phys.vars.iter_mut().zip(&curs).zip(&k3s) {
            *var = cur + h * k;
        }
        let phys = self.eval(t + h);
        let k4s = phys.derivative.clone();

        for ((var, cur), ((k1, k2), (k3, k4))) in phys
            .vars
            .iter_mut()
            .zip(&curs)
            .zip(k1s.iter().zip(&k2s).zip(k3s.iter().zip(&k4s)))
        {
            *var = cur + h * (k1 + 2. * k2 + 2. * k3 + k4) / 6.;
            if !var.is_finite() {
                // Simulation failed, revert
                phys.vars = curs;
                break;
            }
        }

        phys.t += h;
    }

    fn update_anchor();
    fn draw_debug(trans: Mat4);
}
