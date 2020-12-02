use rand::rngs::StdRng;
use std::sync::Arc;

use crate::kdtree::{Bounded, BoundingBox};
pub use cube::Cube;
pub use mesh::{Mesh, Triangle};
pub use monomial_surface::MonomialSurface;
pub use plane::Plane;
pub use sphere::Sphere;

mod cube;
mod mesh;
mod monomial_surface;
mod plane;
mod sphere;

/// Represents a physical shape, which can be hit by a ray to find intersections
pub trait Shape: Send + Sync {
    /// Intersect the shape with a ray, for `t >= t_min`, returning true and mutating
    /// `h` if an intersection was found before the current closest one
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool;

    /// Sample the shape for a random point on its surface, also returning the normal and PDF
    fn sample(&self, target: &glm::DVec3, rng: &mut StdRng) -> (glm::DVec3, glm::DVec3, f64);
}

/// Represents a physical surface, which can compute the nearest point on that shape to a given point
pub trait Physics: Shape {
    /// Find the closest point to a given point
    fn closest_point(&self, point: &glm::DVec3) -> glm::DVec3;
}

impl<T: Shape + ?Sized> Shape for Box<T> {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        self.as_ref().intersect(ray, t_min, record)
    }

    fn sample(&self, target: &glm::DVec3, rng: &mut StdRng) -> (glm::DVec3, glm::DVec3, f64) {
        self.as_ref().sample(target, rng)
    }
}

impl<T: Shape + ?Sized> Shape for Arc<T> {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        self.as_ref().intersect(ray, t_min, record)
    }

    fn sample(&self, target: &glm::DVec3, rng: &mut StdRng) -> (glm::DVec3, glm::DVec3, f64) {
        self.as_ref().sample(target, rng)
    }
}

/// An infinite ray in one direction
#[derive(Copy, Clone)]
pub struct Ray {
    /// The origin of the ray
    pub origin: glm::DVec3,

    /// The unit direction of the ray
    pub dir: glm::DVec3,
}

impl Ray {
    /// Evaluates the ray at a given value of the parameter
    pub fn at(&self, time: f64) -> glm::DVec3 {
        return self.origin + time * self.dir;
    }

    /// Apply a homogeneous transformation to the ray (not normalizing direction)
    pub fn apply_transform(&self, transform: &glm::DMat4) -> Self {
        let origin = transform * (self.origin.to_homogeneous() + glm::vec4(0.0, 0.0, 0.0, 1.0));
        let dir = transform * self.dir.to_homogeneous();
        Self {
            origin: origin.xyz(),
            dir: dir.xyz(),
        }
    }
}

/// Record of when a hit occurs, and the corresponding normal
///
/// TODO: Look into adding more information, such as (u, v) texels
pub struct HitRecord {
    /// The time at which the hit occurs (see `Ray`)
    pub time: f64,

    /// The normal of the hit in some coordinate system
    pub normal: glm::DVec3,
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            time: f64::INFINITY,
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

/// A shape that has been composed with a transformation
pub struct Transformed<T> {
    shape: T,
    transform: glm::DMat4,
    linear: glm::DMat3,
    inverse_transform: glm::DMat4,
    normal_transform: glm::DMat3,
    scale: f64,
}

impl<T> Transformed<T> {
    fn new(shape: T, transform: glm::DMat4) -> Self {
        let inverse_transform = glm::inverse(&transform);
        let linear = glm::mat4_to_mat3(&transform);
        let scale = linear.determinant();
        let normal_transform = glm::inverse_transpose(linear);
        Self {
            shape,
            transform,
            linear,
            inverse_transform,
            normal_transform,
            scale,
        }
    }
}

impl<T: Shape> Shape for Transformed<T> {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        let local_ray = ray.apply_transform(&self.inverse_transform);
        if self.shape.intersect(&local_ray, t_min, record) {
            // Fix normal vectors by multiplying by M^-T
            record.normal = (self.normal_transform * record.normal).normalize();
            true
        } else {
            false
        }
    }

    fn sample(&self, target: &glm::DVec3, rng: &mut StdRng) -> (glm::DVec3, glm::DVec3, f64) {
        let target = (self.inverse_transform * glm::vec4(target.x, target.y, target.z, 1.0)).xyz();
        let (v, n, p) = self.shape.sample(&target, rng);
        let new_normal = (self.normal_transform * n).normalize();
        let parallelepiped_height = (self.linear * n).dot(&new_normal);
        let parallelepiped_base = self.scale / parallelepiped_height;
        (
            (self.transform * glm::vec4(v.x, v.y, v.z, 1.0)).xyz(),
            new_normal,
            p / parallelepiped_base, // divide PDF by the area scale factor
        )
    }
}

impl<T: Bounded> Bounded for Transformed<T> {
    fn bounding_box(&self) -> BoundingBox {
        // This is not necessarily the best bounding box, but it is correct
        let BoundingBox { p_min, p_max } = self.shape.bounding_box();
        let v1 = (self.transform * glm::vec4(p_min.x, p_min.y, p_min.z, 1.0)).xyz();
        let v2 = (self.transform * glm::vec4(p_min.x, p_min.y, p_max.z, 1.0)).xyz();
        let v3 = (self.transform * glm::vec4(p_min.x, p_max.y, p_min.z, 1.0)).xyz();
        let v4 = (self.transform * glm::vec4(p_min.x, p_max.y, p_max.z, 1.0)).xyz();
        let v5 = (self.transform * glm::vec4(p_max.x, p_min.y, p_min.z, 1.0)).xyz();
        let v6 = (self.transform * glm::vec4(p_max.x, p_min.y, p_max.z, 1.0)).xyz();
        let v7 = (self.transform * glm::vec4(p_max.x, p_max.y, p_min.z, 1.0)).xyz();
        let v8 = (self.transform * glm::vec4(p_max.x, p_max.y, p_max.z, 1.0)).xyz();
        BoundingBox {
            p_min: glm::min2(
                &glm::min4(&v1, &v2, &v3, &v4),
                &glm::min4(&v5, &v6, &v7, &v8),
            ),
            p_max: glm::max2(
                &glm::max4(&v1, &v2, &v3, &v4),
                &glm::max4(&v5, &v6, &v7, &v8),
            ),
        }
    }
}

/// An object that can be transformed
pub trait Transformable<T> {
    /// Transform: apply a translation
    fn translate(self, v: &glm::DVec3) -> Transformed<T>;

    /// Transform: apply a scale, in 3 dimensions
    fn scale(self, v: &glm::DVec3) -> Transformed<T>;

    /// Transform: apply a rotation, by an angle in radians about an axis
    fn rotate(self, angle: f64, axis: &glm::DVec3) -> Transformed<T>;

    /// Transform: apply a rotation around the X axis, by an angle in radians
    fn rotate_x(self, angle: f64) -> Transformed<T>;

    /// Transform: apply a rotation around the Y axis, by an angle in radians
    fn rotate_y(self, angle: f64) -> Transformed<T>;

    /// Transform: apply a rotation around the Z axis, by an angle in radians
    fn rotate_z(self, angle: f64) -> Transformed<T>;

    /// Transform: apply a general homogeneous matrix
    fn transform(self, transform: glm::DMat4) -> Transformed<T>;
}

impl<T: Shape> Transformable<T> for T {
    fn translate(self, v: &glm::DVec3) -> Transformed<T> {
        Transformed::new(self, glm::translate(&glm::identity(), v))
    }

    fn scale(self, v: &glm::DVec3) -> Transformed<T> {
        Transformed::new(self, glm::scale(&glm::identity(), v))
    }

    fn rotate(self, angle: f64, axis: &glm::DVec3) -> Transformed<T> {
        Transformed::new(self, glm::rotate(&glm::identity(), angle, axis))
    }

    fn rotate_x(self, angle: f64) -> Transformed<T> {
        Transformed::new(self, glm::rotate_x(&glm::identity(), angle))
    }

    fn rotate_y(self, angle: f64) -> Transformed<T> {
        Transformed::new(self, glm::rotate_y(&glm::identity(), angle))
    }

    fn rotate_z(self, angle: f64) -> Transformed<T> {
        Transformed::new(self, glm::rotate_z(&glm::identity(), angle))
    }

    fn transform(self, transform: glm::DMat4) -> Transformed<T> {
        Transformed::new(self, transform)
    }
}

// This implementation makes it so that chaining transforms doesn't keep nesting into
// the Transformed<Transformed<Transformed<...>>> struct.
impl<T: Shape> Transformed<T> {
    /// Optimized transform: apply a translation
    pub fn translate(self, v: &glm::DVec3) -> Transformed<T> {
        Self::new(
            self.shape,
            glm::translate(&glm::identity(), v) * self.transform,
        )
    }

    /// Optimized transform: apply a scale, in 3 dimensions
    pub fn scale(self, v: &glm::DVec3) -> Transformed<T> {
        Self::new(self.shape, glm::scale(&glm::identity(), v) * self.transform)
    }

    /// Optimized transform: apply a rotation, by an angle in radians about an axis
    pub fn rotate(self, angle: f64, axis: &glm::DVec3) -> Transformed<T> {
        Self::new(
            self.shape,
            glm::rotate(&glm::identity(), angle, axis) * self.transform,
        )
    }

    /// Optimized transform: apply a rotation around the X axis, by an angle in radians
    pub fn rotate_x(self, angle: f64) -> Transformed<T> {
        Self::new(
            self.shape,
            glm::rotate_x(&glm::identity(), angle) * self.transform,
        )
    }

    /// Optimized transform: apply a rotation around the Y axis, by an angle in radians
    pub fn rotate_y(self, angle: f64) -> Transformed<T> {
        Self::new(
            self.shape,
            glm::rotate_y(&glm::identity(), angle) * self.transform,
        )
    }

    /// Optimized transform: apply a rotation around the Z axis, by an angle in radians
    pub fn rotate_z(self, angle: f64) -> Transformed<T> {
        Self::new(
            self.shape,
            glm::rotate_z(&glm::identity(), angle) * self.transform,
        )
    }

    /// Optimized transform: apply a general homogeneous matrix
    pub fn transform(self, transform: glm::DMat4) -> Transformed<T> {
        Self::new(self.shape, transform * self.transform)
    }
}

/// Helper function to construct a sphere
pub fn sphere() -> Sphere {
    Sphere
}

/// Helper function to construct a glass-like monomial surface
pub fn monomial_surface(height: f64, exp: f64) -> MonomialSurface {
    MonomialSurface { height, exp }
}

/// Helper function to construct a plane
pub fn plane(normal: glm::DVec3, value: f64) -> Plane {
    Plane { normal, value }
}

/// Helper function to construct a cube
pub fn cube() -> Cube {
    Cube
}

/// Helper function to construct a simple polygon made from triangles
pub fn polygon(verts: &[glm::DVec3]) -> Mesh {
    let mut tris = Vec::new();
    for i in 1..(verts.len() - 1) {
        tris.push(Triangle::from_vertices(verts[0], verts[i], verts[i + 1]));
    }
    Mesh::new(tris)
}
