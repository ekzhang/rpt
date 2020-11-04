use super::{HitRecord, Ray, Shape};

/// A plane given by linear equation: glm::dot(normal, x) == value
pub struct Plane {
    /// The normal vector
    pub normal: glm::Vec3,

    /// The distance from the origin
    pub value: f32,
}

impl Shape for Plane {
    fn intersect(&self, r: Ray, tmin: f32) -> HitRecord {
        todo!();
    }
}
