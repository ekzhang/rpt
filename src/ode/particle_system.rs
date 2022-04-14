use super::ParticleState;
use crate::shape::MonomialSurface;

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
        for (i, pos_i) in state.pos.iter().enumerate() {
            for (j, pos_j) in state.pos.iter().take(i).enumerate() {
                let dir = glm::normalize(&(pos_i - pos_j));
                let len = glm::length(&(pos_i - pos_j));
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
        let mut acc = vec![glm::vec3(0.0, -1., 0.0); state.pos.len()];
        for (i, pos_i) in state.pos.iter().enumerate() {
            for (j, pos_j) in state.pos.iter().take(i).enumerate() {
                let dir = glm::normalize(&(pos_i - pos_j));
                let len = glm::length(&(pos_i - pos_j));
                if len < 2. * self.radius {
                    let force = -dir * 5. * ((2. * self.radius - len) / self.radius).powi(1);
                    acc[j] += force;
                    acc[j] -= state.vel[j] * 0.5;
                    acc[i] -= force;
                    acc[i] -= state.vel[i] * 0.5;
                }
            }
        }
        let surf = MonomialSurface {
            height: 2.,
            exp:    4.,
        };
        // Surface physics
        for (i, pos_i) in state.pos.iter().enumerate() {
            let closest = surf.closest_point(pos_i);
            let vec = pos_i - closest;
            let normal = glm::normalize(&vec);
            let ratio_intersecting = (self.radius - glm::length(&vec)) / self.radius;
            //            let normal_acc = normal * glm::dot(&acc[i], &normal);
            let normal_vel = state.vel[i].dot(&normal);
            if -0.1 < ratio_intersecting && ratio_intersecting < 0. {
                acc[i] -= 30. * normal * normal_vel.powi(3);
            } else if ratio_intersecting >= 0. {
                acc[i] += 100. * normal * ratio_intersecting.powi(1);
            }
        }
        // Table physics
        for (i, pos_i) in state.pos.iter().enumerate() {
            let normal = glm::vec3(0., 1., 0.);
            let ratio_intersecting = ((self.radius - 0.06) - pos_i.y) / self.radius;
            let normal_vel = state.vel[i].dot(&normal);
            if glm::length(pos_i) > 0.1 {
                // Check that surface normal force does not act on this marble already
                if -0.1 < ratio_intersecting && ratio_intersecting < 0. {
                    acc[i] -= 20. * normal * normal_vel;
                } else if ratio_intersecting >= 0. {
                    acc[i] += 300000. * normal * ratio_intersecting.powi(1);
                }
            }
        }
        for (i, vel_i) in state.vel.iter().enumerate() {
            // introduce "air resistance"
            acc[i] -= vel_i / 5.;
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
