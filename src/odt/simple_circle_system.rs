use crate::{ParticleSystem, ParticleState};

/// System that represents particles moving in a circles
pub struct SimpleCircleSystem;

impl ParticleSystem for SimpleCircleSystem {
    fn time_derivative(&self, state: &ParticleState) -> ParticleState {
        ParticleState {
                pos: state.pos.iter().map(|p| glm::vec3(-p.y,p.x,p.z)).collect(),
                vel: vec![glm::vec3(0.0,0.0,0.0); state.vel.len()]
            }
    }
}
