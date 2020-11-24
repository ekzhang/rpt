use std::ops;

/// Represents current state of a particle system. Includes only positions and velocities currently.
pub struct ParticleState {
    /// Position vectors
    pub pos: Vec<glm::Vec3>,
    /// Velocity vectors
    pub vel: Vec<glm::Vec3>,
}

impl ParticleState {}

impl Clone for ParticleState {
    fn clone(&self) -> Self {
        ParticleState {
            pos: self.pos.clone(),
            vel: self.vel.clone(),
        }
    }
}

impl ops::Add<ParticleState> for ParticleState {
    type Output = ParticleState;

    fn add(self, rhs: ParticleState) -> ParticleState {
        return ParticleState {
            pos: self
                .pos
                .iter()
                .zip(rhs.pos.iter())
                .map(|(x, y)| x + y)
                .collect(),
            vel: self
                .vel
                .iter()
                .zip(rhs.vel.iter())
                .map(|(x, y)| x + y)
                .collect(),
        };
    }
}

impl ops::Mul<f32> for ParticleState {
    type Output = ParticleState;
    fn mul(self, rhs: f32) -> ParticleState {
        return ParticleState {
            pos: self.pos.iter().map(|x| x * rhs).collect(),
            vel: self.vel.iter().map(|x| x * rhs).collect(),
        };
    }
}

impl ops::Div<f32> for ParticleState {
    type Output = ParticleState;
    fn div(self, rhs: f32) -> ParticleState {
        return ParticleState {
            pos: self.pos.iter().map(|x| x / rhs).collect(),
            vel: self.vel.iter().map(|x| x / rhs).collect(),
        };
    }
}

impl ops::Add<ParticleState> for &ParticleState {
    type Output = ParticleState;

    fn add(self, rhs: ParticleState) -> ParticleState {
        return ParticleState {
            pos: self
                .pos
                .iter()
                .zip(rhs.pos.iter())
                .map(|(x, y)| x + y)
                .collect(),
            vel: self
                .vel
                .iter()
                .zip(rhs.vel.iter())
                .map(|(x, y)| x + y)
                .collect(),
        };
    }
}

impl ops::Mul<f32> for &ParticleState {
    type Output = ParticleState;
    fn mul(self, rhs: f32) -> ParticleState {
        return ParticleState {
            pos: self.pos.iter().map(|x| x * rhs).collect(),
            vel: self.vel.iter().map(|x| x * rhs).collect(),
        };
    }
}

impl ops::Div<f32> for &ParticleState {
    type Output = ParticleState;
    fn div(self, rhs: f32) -> ParticleState {
        return ParticleState {
            pos: self.pos.iter().map(|x| x / rhs).collect(),
            vel: self.vel.iter().map(|x| x / rhs).collect(),
        };
    }
}
