/// Represents a glass-like surface, given by height * sqrt(x^2 + z^2)^exp = y, x^2 + z^2 <= 1.
use rand::{rngs::ThreadRng, Rng};
use rand_distr::UnitSphere;

use super::{HitRecord, Ray, Shape};
use crate::kdtree::{Bounded, BoundingBox};

/// A unit sphere centered at the origin
pub struct MonomialSurface {
    pub height: f64,
    pub exp: f64,
}

impl Shape for MonomialSurface {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        /*
        let ray_scaled = Ray {
            origin: glm::vec3(ray.origin.x, ray.origin.y / coef, ray.origin.z),
            dir: glm::vec3(ray.dir.x, ray.dir.y / coef, ray.dir.z),
        };
        */
        let dist = |t: f64| {
            let x = ray.origin.x + t * ray.dir.x;
            let y = ray.origin.y + t * ray.dir.y;
            let z = ray.origin.z + t * ray.dir.z;
            return y - self.height * (x * x + z * z).powf(self.exp / 2f64); // can make exp / 2 integer to speed up
        };
        let maximize: bool = dist(t_min) < 0.0;
        let t_max: f64;
        {
            let mut l: f64 = t_min;
            let mut r: f64 = 10000f64;
            for _ in 0..30 {
                let ml = (2.0 * l + r) / 3.0;
                let mr = (l + 2.0 * r) / 3.0;
                if maximize && dist(ml) < dist(mr) || !maximize && dist(ml) > dist(mr) {
                    l = ml;
                } else {
                    r = mr;
                }
            }
            t_max = l;
        }
        if (dist(t_min) < 0.0) == (dist(t_max) < 0.0) {
            return false;
        }
        let mut l = t_min;
        let mut r = t_max;
        for _ in 0..30 {
            let m = (l + r) / 2.0;
            if (dist(m) >= 0.0) == maximize {
                r = m;
            } else {
                l = m;
            }
        }
        let pos = ray.at(r);
        if pos.x * pos.x + pos.z * pos.z > 1.0 {
            // Check the second equation
            return false;
        }
        record.time = r;

        // TODO: this is valid only for exp = 4, not sure how to do it in general case
        record.normal = glm::normalize(&glm::vec3(
            self.height * 4.0 * pos.x * (pos.x * pos.x + pos.z * pos.z),
            -1.0,
            self.height * 4.0 * pos.z * (pos.x * pos.x + pos.z * pos.z),
        ));

        // The surface is two-sided, so we choose the appropriate normal
        if glm::dot(&record.normal, &ray.dir) > 0.0 {
            record.normal = -record.normal;
        }

        return true;
    }

    fn sample(&self, rng: &mut ThreadRng) -> (glm::DVec3, glm::DVec3, f64) {
        let [x, y, z] = rng.sample(UnitSphere);
        let p = glm::vec3(x, y, z);
        (p, p, 0.25 * std::f64::consts::FRAC_1_PI)
    }
}

impl Bounded for MonomialSurface {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            p_min: glm::vec3(-1.0, 0.0, -1.0),
            p_max: glm::vec3(1.0, self.height, 1.0),
        }
    }
}
