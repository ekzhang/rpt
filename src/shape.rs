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
    /// Intersect the shape with a ray, for `t >= t_min`, returning true and mutating
    /// `h` if an intersection was found before the current closest one
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool;
}

/// An infinite ray in one direction
#[derive(Copy, Clone)]
pub struct Ray {
    /// The origin of the ray
    pub origin: glm::Vec3,

    /// The unit direction of the ray
    pub dir: glm::Vec3,
}

impl Ray {
    /// Evaluates the ray at a given value of the parameter
    pub fn at(&self, time: f32) -> glm::Vec3 {
        return self.origin + time * self.dir;
    }

    /// Apply a homogeneous transformation to the ray (not normalizing direction)
    pub fn apply_transform(&self, transform: &glm::Mat4) -> Self {
        let ref_pt = self.at(1.0);
        let origin = transform * (self.origin.to_homogeneous() + glm::vec4(0.0, 0.0, 0.0, 1.0));
        let origin = glm::vec4_to_vec3(&(origin / origin.w));
        let ref_pt = transform * (ref_pt.to_homogeneous() + glm::vec4(0.0, 0.0, 0.0, 1.0));
        let ref_pt = glm::vec4_to_vec3(&(ref_pt / ref_pt.w));
        Self {
            origin,
            dir: ref_pt - origin,
        }
    }
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

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            time: f32::INFINITY,
            normal: glm::vec3(0.0, 0.0, 0.0),
        }
    }
}

impl HitRecord {
    /// Construct a new `HitRecord` at infinity
    pub fn new() -> Self {
        Default::default()
    }
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
