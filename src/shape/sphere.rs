use super::{HitRecord, Ray, Shape};

/// A unit sphere centered at the origin
pub struct Sphere;

impl Shape for Sphere {
    fn intersect(&self, r: Ray, tmin: f32) -> HitRecord {
        todo!();
    }
}
