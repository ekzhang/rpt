use rand::{rngs::StdRng, Rng};
use rand_distr::UnitCircle;

use super::{HitRecord, Ray, Shape};
use crate::kdtree::{Bounded, BoundingBox};

/// Represents a glass-shaped surface with height and exp parameters
///
/// Points satisfy the relation y = height * sqrt(x^2 + z^2)^exp, x^2 + z^2 <= 1.
///
/// Normals and other things probably can't be generalized, so they work only for exp=4 for now
#[derive(Copy, Clone)]
pub struct MonomialSurface {
    /// The height of the surface
    pub height: f64,
    /// The surface exponent, must be equal to 4 currently
    pub exp: f64,
}

impl Shape for MonomialSurface {
    fn intersect(&self, ray: &Ray, t_min: f64, record: &mut HitRecord) -> bool {
        let (b_min, b_max) = self.bounding_box().intersect(ray);
        if f64::max(b_min, t_min) > f64::min(b_max, record.time) {
            return false;
        }
        let dist = |t: f64| {
            let x = ray.origin.x + t * ray.dir.x;
            let y = ray.origin.y + t * ray.dir.y;
            let z = ray.origin.z + t * ray.dir.z;
            y - self.height * (x * x + z * z).powi(2)
        };
        let coef0 = ray.origin.x.powi(2) + ray.origin.z.powi(2);
        let coef1 = 2. * (ray.origin.x * ray.dir.x + ray.origin.z * ray.dir.z);
        let coef2 = ray.dir.x.powi(2) + ray.dir.z.powi(2);
        let deriv = |t: f64| {
            let dy = 2. * coef0 * coef1
                + 2. * t * (coef1 * coef1 + 2. * coef0 * coef2)
                + 3. * t.powi(2) * 2. * coef1 * coef2
                + 4. * t.powi(3) * coef2 * coef2;
            ray.dir.y - self.height * dy
        };
        let deriv2 = |t: f64| {
            let dy = 2. * (coef1 * coef1 + 2. * coef0 * coef2)
                + 3. * 2. * t * 2. * coef1 * coef2
                + 4. * 3. * t.powi(2) * coef2 * coef2;
            -self.height * dy
        };
        let t_max;
        let maximize: bool = dist(t_min) < 0.0;
        if maximize {
            let mut cur_x = (b_min + b_max) / 2.;
            for _ in 0..10 {
                let f = dist(cur_x);
                if f > 0. {
                    break;
                }
                let der = deriv(cur_x);
                let der2 = deriv2(cur_x);
                cur_x -= der / der2;
            }
            if dist(cur_x) < 0. && deriv(cur_x).abs() > 1e-4 {
                println!("{}", deriv(cur_x).abs());
            }
            t_max = cur_x;
            if t_max < t_min {
                return false;
            }
        } else {
            t_max = 10000.;
        }
        if (dist(t_min) < 0.0) == (dist(t_max) < 0.0) {
            return false;
        }
        let mut l = t_min;
        let mut r = t_max;
        for _ in 0..60 {
            let m = (l + r) / 2.0;
            if (dist(m) >= 0.0) == maximize {
                r = m;
            } else {
                l = m;
            }
        }
        if r > record.time {
            return false;
        }
        let pos = ray.at(r);
        if pos.x * pos.x + pos.z * pos.z > 1.0 {
            // Check the second equation
            return false;
        }
        record.time = r;

        record.normal = glm::normalize(&glm::vec3(
            self.height * 4.0 * pos.x * (pos.x * pos.x + pos.z * pos.z),
            -1.0,
            self.height * 4.0 * pos.z * (pos.x * pos.x + pos.z * pos.z),
        ));

        // The surface is two-sided, so we choose the appropriate normal
        if glm::dot(&record.normal, &ray.dir) > 0.0 {
            record.normal = -record.normal;
        }

        true
    }

    fn sample(&self, _target: &glm::DVec3, rng: &mut StdRng) -> (glm::DVec3, glm::DVec3, f64) {
        let [x, z]: [f64; 2] = rng.sample(UnitCircle);
        let pos = glm::vec3(x, self.height * (x * x + z * z).powf(self.exp / 2.), z);
        let mut normal = glm::normalize(&glm::vec3(
            self.height * 4. * pos.x * (pos.x * pos.x + pos.z * pos.z),
            -1.,
            self.height * 4. * pos.z * (pos.x * pos.x + pos.z * pos.z),
        ));
        // Again, only valid for exp = 4
        const AREA: f64 = 6.3406654362; // thanks WolframAlpha, hope I have set up the integrals correctly
        if rng.gen::<bool>() {
            normal = -normal;
        }
        (pos, normal, 1. / (2. * AREA)) // 2 * AREA because there are two sides
    }
}

impl MonomialSurface {
    /// Estimates the closest point on the surface to a given point
    pub fn closest_point(&self, point: &glm::DVec3) -> glm::DVec3 {
        if glm::length(point) < 1e-12 {
            // Can't normalize in this case
            return *point;
        }
        // Move to the 2d coordinate system
        let px = point.x.hypot(point.z);
        let py = point.y;
        let pt = glm::vec2(px, py);
        let mut res = (1e18, -1.);
        for x in -100..101 {
            let xf = x as f64 / 100.;
            let dist2 = glm::distance2(&pt, &glm::vec2(xf, self.height * xf.powi(4)));
            if dist2 < res.0 {
                res = (dist2, xf);
            }
        }
        let xz = res.1 * glm::normalize(&glm::vec2(point.x, point.z));
        glm::vec3(
            xz.x,
            self.height * (xz.x.powi(2) + xz.y.powi(2)).powi(2),
            xz.y,
        )
    }

    /// More precise and slower version of the closest_point function
    pub fn closest_point_precise(&self, point: &glm::DVec3) -> glm::DVec3 {
        if glm::length(point) < 1e-12 {
            // Can't normalize in this case
            return *point;
        }
        // Move to the 2d coordinate system
        let px = point.x.hypot(point.z);
        let py = point.y;
        let pt = glm::vec2(px, py);
        let mut res = (1e18, -1.);
        for x in -10000..10001 {
            let xf = x as f64 / 10000.;
            let dist2 = glm::distance2(&pt, &glm::vec2(xf, self.height * xf.powi(4)));
            if dist2 < res.0 {
                res = (dist2, xf);
            }
        }
        let xz = res.1 * glm::normalize(&glm::vec2(point.x, point.z));
        glm::vec3(
            xz.x,
            self.height * (xz.x.powi(2) + xz.y.powi(2)).powi(2),
            xz.y,
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monomial_closest_point_works() {
        let surf = MonomialSurface {
            height: 1.,
            exp: 4.,
        };
        let test_xz = |x: f64, z: f64| {
            // Test the closest point to (x, y(x, z), z)
            let pt = glm::vec3(x, (x.powi(2) + z.powi(2)).powi(2), z);
            assert!(glm::distance(&pt, &surf.closest_point(&pt)) < 0.03);
        };
        let test_xy = |x: f64, y: f64| {
            // Test the closest point to the given (x, y, 0) (assume z=0 because the problem is the same in 3D)
            let pt = glm::vec3(x, y, 0.);
            let closest = surf.closest_point(&pt);
            let dist = glm::distance2(&pt, &closest);
            for i in -100..100 {
                let xi = i as f64 / 100.;
                let dist1 = glm::distance2(&pt, &glm::vec3(xi, xi.powi(4), 0.));
                if dist1 < dist - 1e-5 {
                    println!("Test failed for ({}, {}): found distance {}, but there is a point within distance {}", x, y, dist.sqrt(), dist1.sqrt());
                    println!("The closest point was ({}, {})", closest.x, closest.y);
                }
            }
        };
        test_xz(0.0, 1.0);
        test_xz(0.0, -1.0);
        test_xz(0.23234, 0.723423);
        test_xz(0.12323, -0.23423);
        test_xz(0.0, 0.00001);
        test_xz(0.0, -0.00001);
        for i in 1..10000 {
            test_xz(0.0, i as f64 / 10000.0);
            test_xz(0.0, -i as f64 / 10000.0);
        }
        test_xz(0.0, 0.0);
        test_xz(0.0, 1e-13);
        test_xz(0.0, 1e-12);
        test_xz(0.0, 1e-11);
        test_xz(0.0, 1e-10);

        test_xy(0., 1.);
        test_xy(0.123, 0.3124);
        test_xy(-0.123, 0.4123);
        test_xy(0., -1.);
        test_xy(0., -10.);
        test_xy(-1., 2.);
        test_xy(-1., 0.5);
    }
}
