use super::{HitRecord, Ray, Shape};

/// A plane given by linear equation: glm::dot(normal, x) == value
pub struct Plane {
    /// The normal vector
    pub normal: glm::Vec3,

    /// The distance from the origin
    pub value: f32,
}

impl Shape for Plane {
    /// Ray-plane intersection
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool {
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
}
