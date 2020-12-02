use std::ops;

/// Represents current state of a particle system, including positions and velocities
#[derive(Clone, Debug)]
pub struct ParticleState {
    /// Position vectors
    pub pos: Vec<glm::DVec3>,
    /// Velocity vectors
    pub vel: Vec<glm::DVec3>,
}

fn add_slices(a: &[glm::DVec3], b: &[glm::DVec3]) -> Vec<glm::DVec3> {
    <_>::zip(a.iter(), b.iter()).map(|(x, y)| x + y).collect()
}

macro_rules! impl_add {
    ($rhs:ty, $fortype:ty) => {
        impl ops::Add<$rhs> for $fortype {
            type Output = ParticleState;

            fn add(self, rhs: $rhs) -> ParticleState {
                return ParticleState {
                    pos: add_slices(&self.pos, &rhs.pos),
                    vel: add_slices(&self.vel, &rhs.vel),
                };
            }
        }
    };
}

impl_add!(ParticleState, ParticleState);
impl_add!(&ParticleState, ParticleState);
impl_add!(ParticleState, &ParticleState);
impl_add!(&ParticleState, &ParticleState);

macro_rules! impl_scalar_arithmetic {
    ($trait:tt, $fortype:ty, $fn:ident, $op:tt) => {
        impl ops::$trait<f64> for $fortype {
            type Output = ParticleState;

            fn $fn(self, rhs: f64) -> ParticleState {
                ParticleState {
                    pos: self.pos.iter().map(|x| x $op rhs).collect(),
                    vel: self.vel.iter().map(|x| x $op rhs).collect(),
                }
            }
        }
    }
}

impl_scalar_arithmetic!(Mul, ParticleState, mul, *);
impl_scalar_arithmetic!(Mul, &ParticleState, mul, *);

impl_scalar_arithmetic!(Div, ParticleState, div, /);
impl_scalar_arithmetic!(Div, &ParticleState, div, /);
