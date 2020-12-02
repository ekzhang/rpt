use rand::{rngs::StdRng, Rng};
use rand_distr::UnitDisc;

use crate::shape::Ray;

/// A simple thin-lens perspective camera
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: glm::DVec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f64,

    /// Aperture radius for depth-of-field effects
    pub aperture: f64,

    /// Focal distance, if aperture radius is nonzero
    pub focal_distance: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            fov: std::f64::consts::FRAC_PI_6,
            aperture: 0.0,
            focal_distance: 0.0,
        }
    }
}

impl Camera {
    /// Perspective camera looking at a point, with a given field of view
    pub fn look_at(eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3, fov: f64) -> Self {
        let direction = (center - eye).normalize();
        let up = (up - up.dot(&direction) * direction).normalize();
        Self {
            eye,
            direction,
            up,
            fov,
            aperture: 0.0,
            focal_distance: 0.0,
        }
    }

    /// Focus the camera on a position, with simulated depth-of-field
    pub fn focus(mut self, focal_point: glm::DVec3, aperture: f64) -> Self {
        self.focal_distance = (focal_point - self.eye).dot(&self.direction);
        self.aperture = aperture;
        self
    }

    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    pub fn cast_ray(&self, x: f64, y: f64, rng: &mut StdRng) -> Ray {
        // cot(f / 2) = depth / radius
        let d = (self.fov / 2.0).tan().recip();
        let right = glm::cross(&self.direction, &self.up).normalize();
        let mut origin = self.eye;
        let mut new_dir = d * self.direction + x * right + y * self.up;
        if self.aperture > 0.0 {
            // Depth of field
            let focal_point = origin + new_dir.normalize() * self.focal_distance;
            let [x, y]: [f64; 2] = rng.sample(UnitDisc);
            origin += (x * right + y * self.up) * self.aperture;
            new_dir = focal_point - origin;
        }
        Ray {
            origin,
            dir: new_dir.normalize(),
        }
    }
}
