use rand::{distributions::Uniform, rngs::ThreadRng, Rng};

use super::{HitRecord, Ray, Shape};
use crate::kdtree::{Bounded, BoundingBox};

/// A unit cube centered at the origin
pub struct Cube;

impl Bounded for Cube {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            p_min: glm::vec3(-0.5, -0.5, -0.5),
            p_max: glm::vec3(0.5, 0.5, 0.5),
        }
    }
}

impl Shape for Cube {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        let compute_interval = |dim: usize| {
            let mut x1 = (-0.5_f64 - ray.origin[dim]) / ray.dir[dim];
            let mut x2 = (0.5_f64 - ray.origin[dim]) / ray.dir[dim];
            let mut x1n: glm::DVec3 = glm::zero();
            let mut x2n: glm::DVec3 = glm::zero();
            x1n[dim] = -1.0;
            x2n[dim] = 1.0;
            if x1 > x2 {
                std::mem::swap(&mut x1, &mut x2);
                std::mem::swap(&mut x1n, &mut x2n);
            }
            (x1, x2, x1n, x2n)
        };
        let (x1, x2, x1n, x2n) = compute_interval(0);
        let (y1, y2, y1n, y2n) = compute_interval(1);
        let (z1, z2, z1n, z2n) = compute_interval(2);

        let (start, start_normal) = {
            if x1 > y1 && x1 > z1 {
                (x1, x1n)
            } else if y1 > z1 {
                (y1, y1n)
            } else {
                (z1, z1n)
            }
        };
        let (end, end_normal) = {
            if x2 < y2 && x2 < z2 {
                (x2, x2n)
            } else if y2 < z2 {
                (y2, y2n)
            } else {
                (z2, z2n)
            }
        };

        if start > end || end < t_min {
            return false;
        }
        let (time, normal) = if start < t_min {
            (end, end_normal)
        } else {
            (start, start_normal)
        };
        if time < record.time {
            record.time = time;
            record.normal = normal;
            true
        } else {
            false
        }
    }

    fn sample(&self, rng: &mut ThreadRng) -> (glm::DVec3, glm::DVec3, f64) {
        let a = rng.gen::<f64>() - 0.5;
        let b = rng.gen::<f64>() - 0.5;
        let (v, n) = match rng.sample(Uniform::from(0..6)) {
            0 => (glm::vec3(a, b, 0.5), glm::vec3(0.0, 0.0, 1.0)),
            1 => (glm::vec3(a, b, -0.5), glm::vec3(0.0, 0.0, -1.0)),
            2 => (glm::vec3(a, 0.5, b), glm::vec3(0.0, 1.0, 0.0)),
            3 => (glm::vec3(a, -0.5, b), glm::vec3(0.0, -1.0, 0.0)),
            4 => (glm::vec3(0.5, a, b), glm::vec3(1.0, 0.0, 0.0)),
            5 => (glm::vec3(-0.5, a, b), glm::vec3(-1.0, 0.0, 0.0)),
            _ => unreachable!(),
        };
        (v, n, 1.0 / 6.0)
    }
}
