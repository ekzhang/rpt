use crate::{ParticleSystem, ParticleState};

/// Implements Runge-Kutta 4 integrator
// Not much OOP yet, not sure if we want it

/// Do one integration step, a helper function for RK4
fn integrate_step(system: &impl ParticleSystem, state: ParticleState, step: f32) -> ParticleState {
    let k1 = system.time_derivative(&state);
    let k2 = system.time_derivative(&(&state + &k1 * (step / 2f32)));
    let k3 = system.time_derivative(&(&state + &k2 * (step / 2f32)));
    let k4 = system.time_derivative(&(&state + &k3 * step));
    return state + (k1 + k2 * 2f32 + k3 * 2f32 + k4) * (step / 6f32);
}

/// Simulate a process for given time, with given time step
pub fn rk4_integrate(system: &impl ParticleSystem, state: ParticleState, mut time: f32, step: f32) -> ParticleState {
    let mut res = state;
    while time > step {
        res = integrate_step(system, res, step);
        time -= step;
    }
    res = integrate_step(system, res, time);
    return res;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::odt::simple_circle_system::SimpleCircleSystem;

    #[test]
    fn rk4_works() {
        let state = ParticleState {
            pos: vec![glm::vec3(1.0, 0.0, 0.0)],
            vel: vec![glm::vec3(0.0, 0.0, 0.0)]
        };
        let res = rk4_integrate(&SimpleCircleSystem{}, state, std::f32::consts::TAU, 0.005);
        println!("{}", res.pos[0].x);
        assert!(glm::distance(&res.pos[0], &glm::vec3(1.0, 0.0, 0.0)) < 1e-3);
    }
}