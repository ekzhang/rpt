use rand::rngs::ThreadRng;

use super::{HitRecord, Ray, Shape};

/// A plane represented by the linear equation x • normal = value
pub struct Plane {
    /// The normal vector
    pub normal: glm::DVec3,

    /// The distance from the origin
    pub value: f64,
}

impl Shape for Plane {
    /// Ray-plane intersection
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        let cosine = self.normal.dot(&ray.dir);
        if cosine.abs() < 1e-8 {
            // Parallel ray and plane
            return false;
        }

        let time = (self.value - self.normal.dot(&ray.origin)) / cosine;
        if time >= t_min && time < record.time {
            record.time = time;
            record.normal = -self.normal.normalize() * cosine.signum();
            true
        } else {
            false
        }
    }

    fn sample(&self, _target: &glm::DVec3, _rng: &mut ThreadRng) -> (glm::DVec3, glm::DVec3, f64) {
        unimplemented!()
    }
}
