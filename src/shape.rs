// TODO: Constructive solid geometry
pub use mesh::{Mesh, Triangle};
pub use plane::Plane;
pub use sphere::Sphere;
use std::rc::Rc;

mod mesh;
mod plane;
mod sphere;

/// Represents a physical shape, which can be hit by a ray to find intersections
pub trait Shape {
    /// Intersect the shape with a ray, for t >= tmin
    fn intersect(&self, r: Ray, tmin: f32) -> HitRecord;
}

/// An infinite ray in one direction
pub struct Ray {
    /// The origin of the ray
    pub origin: glm::Vec3,

    /// The unit direction of the ray
    pub dir: glm::Vec3,
}

/// Record of when a hit occurs, and the corresponding normal
///
/// TODO: Look into adding more information, such as (u, v) texels
pub struct HitRecord {
    /// The time at which the hit occurs (see `Ray`)
    pub time: f32,

    /// The normal of the hit in some coordinate system
    pub normal: glm::Vec3,
}

/// Helper function to construct an `Rc` for a sphere
pub fn sphere() -> Rc<Sphere> {
    Rc::new(Sphere)
}

/// Helper function to construct an `Rc` for a plane
pub fn plane(normal: glm::Vec3, value: f32) -> Rc<Plane> {
    Rc::new(Plane { normal, value })
}

/// Helper function to load a mesh from an STL .OBJ file
pub fn load_obj(path: &str) -> color_eyre::Result<Rc<Mesh>> {
    todo!();
}
