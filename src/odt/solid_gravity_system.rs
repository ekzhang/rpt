use crate::{ParticleSystem, ParticleState};

/// System that represents solid gravity objects in space
pub struct SolidGravitySystem;

impl ParticleSystem for SolidGravitySystem {
    fn time_derivative(&self, state: &ParticleState) -> ParticleState {
        let mut acc = vec![glm::vec3(0.0, 0.0, 0.0); state.pos.len()];
        for i in 0..state.pos.len() {
            for j in 0..i {
                let dir = glm::normalize(&(state.pos[i] - state.pos[j]));
                let len = glm::length(&(state.pos[i] - state.pos[j]));
                let force = dir * (len.powf(-2.0) - 0.0001 * len.powf(-5.0));
                acc[j] += force;
                acc[i] -= force;
            }
        }

        return ParticleState {pos: state.vel.clone(), vel: acc}
    }
}