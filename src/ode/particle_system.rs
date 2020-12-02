use super::ParticleState;
use crate::{MonomialSurface, Physics};

/// A trait that represents a system formulating some physical laws
pub trait ParticleSystem {
    /// Compute time-derivative of a state
    fn time_derivative(&self, state: &ParticleState) -> ParticleState;

    /// Integrate the system with RK4 for a given time, with given time step
    fn rk4_integrate(&self, state: &mut ParticleState, mut time: f64, step: f64) {
        // Do one integration step, a helper function for RK4
        let mut integrate_step = |step: f64| {
            let k1 = self.time_derivative(state);
            let k2 = self.time_derivative(&(&*state + &k1 * (step / 2.0)));
            let k3 = self.time_derivative(&(&*state + &k2 * (step / 2.0)));
            let k4 = self.time_derivative(&(&*state + &k3 * step));
            *state = &*state + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (step / 6.0);
        };
        while time > step {
            integrate_step(step);
            time -= step;
        }
        integrate_step(time);
    }
}

pub struct SimpleCircleSystem;

impl ParticleSystem for SimpleCircleSystem {
    fn time_derivative(&self, state: &ParticleState) -> ParticleState {
        ParticleState {
            pos: state
                .pos
                .iter()
                .map(|p| glm::vec3(-p.y, p.x, 0.0))
                .collect(),
            vel: vec![glm::vec3(0.0, 0.0, 0.0); state.vel.len()],
        }
    }
}

/// System that represents solid gravity objects in space
pub struct SolidGravitySystem;

impl ParticleSystem for SolidGravitySystem {
    fn time_derivative(&self, state: &ParticleState) -> ParticleState {
        let mut acc = vec![glm::vec3(0.0, 0.0, 0.0); state.pos.len()];
        for i in 0..state.pos.len() {
            for j in 0..i {
                let dir = glm::normalize(&(state.pos[i] - state.pos[j]));
                let len = glm::length(&(state.pos[i] - state.pos[j]));
                let force = dir * (len.powi(-2) - 0.0001 * len.powi(-5));
                acc[j] += force;
                acc[i] -= force;
            }
        }

        ParticleState {
            pos: state.vel.clone(),
            vel: acc,
        }
    }
}

/// System that represents marbles and a glass
pub struct MarblesSystem {
    /// Radius of each of the marbles in the system
    pub radius: f64,
}

impl ParticleSystem for MarblesSystem {
    fn time_derivative(&self, state: &ParticleState) -> ParticleState {
        let mut acc = vec![glm::vec3(0.0, -0.5, 0.0); state.pos.len()];
        for i in 0..state.pos.len() {
            for j in 0..i {
                let dir = glm::normalize(&(state.pos[i] - state.pos[j]));
                let len = glm::length(&(state.pos[i] - state.pos[j]));
                if len < 2. * self.radius {
                    let force = -dir * 5. * ((2. * self.radius - len) / self.radius).powi(1);
                    acc[j] += force;
                    acc[i] -= force;
                }
            }
        }
        let surf = MonomialSurface {
            height: 2.,
            exp: 4.,
        };
        for i in 0..state.pos.len() {
            let closest = surf.closest_point(&state.pos[i]);
            let vec = state.pos[i] - closest;
            let normal = glm::normalize(&vec);
            let ratio_intersecting = (self.radius - glm::length(&vec)) / self.radius;
            //            let normal_acc = normal * glm::dot(&acc[i], &normal);
            let normal_vel = &state.vel[i].dot(&normal);
            if -0.1 < ratio_intersecting && ratio_intersecting < 0. {
                acc[i] -= 30. * normal * normal_vel.powi(3);
            } else if ratio_intersecting >= 0. {
                acc[i] += 100. * normal * ratio_intersecting.powi(1);
            }
        }

        ParticleState {
            pos: state.vel.clone(),
            vel: acc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk4_works() {
        let mut state = ParticleState {
            pos: vec![glm::vec3(1.0, 0.0, 0.0)],
            vel: vec![glm::vec3(0.0, 0.0, 0.0)],
        };
        SimpleCircleSystem.rk4_integrate(&mut state, std::f64::consts::TAU, 0.005);
        assert!(glm::distance(&state.pos[0], &glm::vec3(1.0, 0.0, 0.0)) < 1e-3);
        state = ParticleState {
            pos: vec![glm::vec3(1.0, 0.0, 0.0)],
            vel: vec![glm::vec3(0.0, 0.0, 0.0)],
        };
        SimpleCircleSystem.rk4_integrate(&mut state, std::f64::consts::PI, 0.005);
        assert!(glm::distance(&state.pos[0], &glm::vec3(-1.0, 0.0, 0.0)) < 1e-3);
    }
}
