use rand::{rngs::ThreadRng, Rng};

use super::{HitRecord, Ray, Shape};
use crate::kdtree::{Bounded, BoundingBox, KdTree};

/// A triangle with three vertices and three normals
pub struct Triangle {
    /// The first vertex
    pub v1: glm::DVec3,
    /// The second vertex
    pub v2: glm::DVec3,
    /// The third vertex
    pub v3: glm::DVec3,

    /// The first normal vector
    pub n1: glm::DVec3,
    /// The second normal vector
    pub n2: glm::DVec3,
    /// The third normal vector
    pub n3: glm::DVec3,
}

impl Triangle {
    /// Construct a triangle from three vertices, inferring the normals
    pub fn from_vertices(v1: glm::DVec3, v2: glm::DVec3, v3: glm::DVec3) -> Self {
        let n = (v2 - v1).cross(&(v3 - v1)).normalize();
        Self {
            v1,
            v2,
            v3,
            n1: n,
            n2: n,
            n3: n,
        }
    }
}

impl Bounded for Triangle {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            p_min: glm::min3(&self.v1, &self.v2, &self.v3),
            p_max: glm::max3(&self.v1, &self.v2, &self.v3),
        }
    }
}

impl Shape for Triangle {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        let (d0, d1) = (self.v2 - self.v1, self.v3 - self.v1);
        let plane_normal = d0.cross(&d1).normalize();
        let cosine = plane_normal.dot(&ray.dir);
        if cosine.abs() < 1e-8 {
            // Parallel ray and plane of triangle
            return false;
        }
        let time = plane_normal.dot(&(self.v1 - ray.origin)) / cosine;
        if time < t_min || time >= record.time {
            return false;
        }

        // Okay, so let's compute barycentric coordinates now, fast
        // https://gamedev.stackexchange.com/a/23745
        let d2 = ray.at(time) - self.v1;
        let d00 = d0.dot(&d0);
        let d01 = d0.dot(&d1);
        let d11 = d1.dot(&d1);
        let d20 = d2.dot(&d0);
        let d21 = d2.dot(&d1);
        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
            record.time = time;
            record.normal = (u * self.n1 + v * self.n2 + w * self.n3).normalize();
            true
        } else {
            false
        }
    }

    fn sample(&self, _target: &glm::DVec3, rng: &mut ThreadRng) -> (glm::DVec3, glm::DVec3, f64) {
        let mut u: f64 = rng.gen();
        let mut v: f64 = rng.gen();
        while u + v > 1.0 {
            u = rng.gen();
            v = rng.gen();
        }
        let w = 1.0 - u - v;
        let area: f64 = 0.5 * (self.v2 - self.v1).cross(&(self.v3 - self.v1)).magnitude();
        (
            u * self.v1 + v * self.v2 + w * self.v3,
            (u * self.n1 + v * self.n2 + w * self.n3).normalize(),
            area.recip(),
        )
    }
}

/// A triangle mesh, stored using a kd-tree
pub type Mesh = KdTree<Triangle>;
