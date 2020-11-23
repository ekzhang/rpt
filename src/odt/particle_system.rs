use crate::ParticleState;

/// A trait that represents a system formulating some physical laws
pub trait ParticleSystem {
    /// Compute gradient of a state
    fn time_derivative(&self, state: &ParticleState) -> ParticleState;
}