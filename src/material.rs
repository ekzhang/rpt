use rand::{rngs::StdRng, Rng};
use rand_distr::{UnitCircle, UnitDisc};

use crate::color::{hex_color, Color};

/// Represents a shader material with some physical properties
#[derive(Copy, Clone)]
pub struct Material {
    /// Albedo color
    pub color: Color,

    /// Index of refraction
    pub index: f64,

    /// Roughness parameter for Beckmann microfacet distribution
    pub roughness: f64,

    /// Metallic versus dielectric
    pub metallic: f64,

    /// Self-emittance of light
    pub emittance: f64,

    /// Transmittance (e.g., glass)
    pub transparent: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self::specular(hex_color(0xff0000), 0.5) // red
    }
}

impl Material {
    /// Perfect diffuse (Lambertian) material with a given color
    pub fn diffuse(color: Color) -> Material {
        Material {
            color,
            index: 1.5,
            roughness: 1.0,
            metallic: 0.0,
            emittance: 0.0,
            transparent: false,
        }
    }

    /// Specular material with a given color and roughness
    pub fn specular(color: Color, roughness: f64) -> Material {
        Material {
            color,
            index: 1.5,
            roughness,
            metallic: 0.0,
            emittance: 0.0,
            transparent: false,
        }
    }

    /// Clear material with a specified index of refraction and roughness (such as glass)
    pub fn clear(index: f64, roughness: f64) -> Material {
        Material {
            color: glm::vec3(1.0, 1.0, 1.0),
            index,
            roughness,
            metallic: 0.0,
            emittance: 0.0,
            transparent: true,
        }
    }

    /// Colored transparent material
    pub fn transparent(color: Color, index: f64, roughness: f64) -> Material {
        Material {
            color,
            index,
            roughness,
            metallic: 0.0,
            emittance: 0.0,
            transparent: true,
        }
    }

    /// Metallic material (has extra tinted specular reflections)
    pub fn metallic(color: Color, roughness: f64) -> Material {
        Material {
            color,
            index: 1.5,
            roughness,
            metallic: 1.0,
            emittance: 0.0,
            transparent: false,
        }
    }

    /// Perfect emissive material, useful for modeling area lights
    pub fn light(color: Color, emittance: f64) -> Material {
        Material {
            color,
            index: 1.0,
            roughness: 1.0,
            metallic: 0.0,
            emittance,
            transparent: false,
        }
    }
}

#[allow(clippy::many_single_char_names)]
impl Material {
    /// Bidirectional scattering distribution function
    ///
    /// - `n` - surface normal vector
    /// - `wo` - unit direction vector toward the viewer
    /// - `wi` - unit direction vector toward the incident ray
    ///
    /// This works for both opaque and transmissive materials, based on a Beckmann
    /// microfacet distribution model, Cook-Torrance shading for the specular component,
    /// and Lambertian shading for the diffuse component. Useful references:
    ///
    /// - http://www.codinglabs.net/article_physically_based_rendering_cook_torrance.aspx
    /// - https://computergraphics.stackexchange.com/q/4394
    /// - https://graphics.stanford.edu/courses/cs148-10-summer/docs/2006--degreve--reflection_refraction.pdf
    /// - http://www.pbr-book.org/3ed-2018/Materials/BSDFs.html
    /// - https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf
    pub fn bsdf(&self, n: &glm::DVec3, wo: &glm::DVec3, wi: &glm::DVec3) -> Color {
        let n_dot_wi = n.dot(wi);
        let n_dot_wo = n.dot(wo);
        let wi_outside = n_dot_wi.is_sign_positive();
        let wo_outside = n_dot_wo.is_sign_positive();
        if !self.transparent && (!wi_outside || !wo_outside) {
            // Opaque materials do not transmit light
            return glm::vec3(0.0, 0.0, 0.0);
        }
        if wi_outside == wo_outside {
            let h = (wi + wo).normalize(); // halfway vector
            let wo_dot_h = wo.dot(&h);
            let n_dot_h = n.dot(&h);
            let nh2 = n_dot_h.powi(2);

            // d: microfacet distribution function
            // D = exp(((n • h)^2 - 1) / (m^2 (n • h)^2)) / (π m^2 (n • h)^4)
            let m2 = self.roughness * self.roughness;
            let d = ((nh2 - 1.0) / (m2 * nh2)).exp() / (m2 * glm::pi::<f64>() * nh2 * nh2);

            // f: fresnel, schlick's approximation
            // F = F0 + (1 - F0)(1 - wi • h)^5
            let f = if !wi_outside && (1.0 - wo_dot_h * wo_dot_h).sqrt() * self.index > 1.0 {
                // Total internal reflection
                glm::vec3(1.0, 1.0, 1.0)
            } else {
                let f0 = ((self.index - 1.0) / (self.index + 1.0)).powi(2);
                let f0 = glm::lerp(&glm::vec3(f0, f0, f0), &self.color, self.metallic);
                f0 + (glm::vec3(1.0, 1.0, 1.0) - f0) * (1.0 - wo_dot_h).powi(5)
            };

            // g: geometry function, microfacet shadowing
            // G = min(1, 2(n • h)(n • wo)/(wo • h), 2(n • h)(n • wi)/(wo • h))
            let g = f64::min(n_dot_wi * n_dot_h, n_dot_wo * n_dot_h);
            let g = (2.0 * g) / wo_dot_h;
            let g = g.min(1.0);

            // BRDF: putting it all together
            // Cook-Torrance = DFG / (4(n • wi)(n • wo))
            // Lambert = (1 - F) * c / π
            let specular = d * f * g / (4.0 * n_dot_wo * n_dot_wi);
            if self.transparent {
                specular
            } else {
                let diffuse =
                    (glm::vec3(1.0, 1.0, 1.0) - f).component_mul(&self.color) / glm::pi::<f64>();
                specular + diffuse
            }
        } else {
            // Ratio of refractive indices, η_i / η_o
            let eta_t = if wo_outside {
                self.index
            } else {
                1.0 / self.index
            };
            let h = (wi * eta_t + wo).normalize(); // halfway vector
            let wi_dot_h = wi.dot(&h);
            let wo_dot_h = wo.dot(&h);
            let n_dot_h = n.dot(&h);
            let nh2 = n_dot_h.powi(2);

            // d: microfacet distribution function
            // D = exp(((n • h)^2 - 1) / (m^2 (n • h)^2)) / (π m^2 (n • h)^4)
            let m2 = self.roughness * self.roughness;
            let d = ((nh2 - 1.0) / (m2 * nh2)).exp() / (m2 * glm::pi::<f64>() * nh2 * nh2);

            // f: fresnel, schlick's approximation
            // F = F0 + (1 - F0)(1 - wi • h)^5
            let f0 = ((self.index - 1.0) / (self.index + 1.0)).powi(2);
            let f0 = glm::lerp(&glm::vec3(f0, f0, f0), &self.color, self.metallic);
            let f = f0 + (glm::vec3(1.0, 1.0, 1.0) - f0) * (1.0 - wi_dot_h.abs()).powi(5);

            // g: geometry function, microfacet shadowing
            // G = min(1, 2(n • h)(n • wo)/(wo • h), 2(n • h)(n • wi)/(wo • h))
            let g = f64::min((n_dot_wi * n_dot_h).abs(), (n_dot_wo * n_dot_h).abs());
            let g = (2.0 * g) / wo_dot_h.abs();
            let g = g.min(1.0);

            // BTDF: putting it all together
            // Cook-Torrance = |h • wi|/|n • wi| * |h • wo|/|n • wo|
            //                  * η_o^2 (1 - F)DG / (η_i (h • wi) + η_o (h • wo))^2
            let btdf = (wi_dot_h * wo_dot_h / (n_dot_wi * n_dot_wo)).abs()
                * (d * (glm::vec3(1.0, 1.0, 1.0) - f) * g / (eta_t * wi_dot_h + wo_dot_h).powi(2));
            btdf.component_mul(&self.color)
        }
    }

    /// Sample the light hemisphere, returning a tuple of (direction vector, PDF)
    ///
    /// This implementation samples according to the Beckmann distribution
    /// function D. Specifically, it uses the fact that ∫ D(h) (n • h) dω = 1,
    /// which creates a probability distribution that can be sampled from using a
    /// probability integral transform.
    ///
    /// We also need to sample from the diffuse BRDF as well, independently. We
    /// calculate the ratio of samples from the diffuse vs specular components by
    /// estimating the average magnitude of the Fresnel term.
    ///
    /// Reference: https://agraphicsguy.wordpress.com/2015/11/01/sampling-microfacet-brdf/
    pub fn sample_f(
        &self,
        n: &glm::DVec3,
        wo: &glm::DVec3,
        rng: &mut StdRng,
    ) -> Option<(glm::DVec3, f64)> {
        let m2 = self.roughness * self.roughness;

        // Estimate specular contribution using Fresnel term
        let f0 = ((self.index - 1.0) / (self.index + 1.0)).powi(2);
        let f = (1.0 - self.metallic) * f0 + self.metallic * self.color.mean();
        let f = glm::mix_scalar(f, 1.0, 0.2);

        // Ratio of refractive indices
        let eta_t = if wo.dot(n) > 0.0 {
            self.index
        } else {
            1.0 / self.index
        };

        let beckmann = |rng: &mut StdRng| {
            // PIT for Beckmann distribution microfacet normal
            // θ = arctan √(-m^2 ln U)
            let theta = (m2 * -rng.gen::<f64>().ln()).sqrt().atan();
            let (sin_t, cos_t) = theta.sin_cos();

            // Generate halfway vector by sampling azimuth uniformly
            let [x, y]: [f64; 2] = rng.sample(UnitCircle);
            let h = glm::vec3(x * sin_t, y * sin_t, cos_t);
            local_to_world(n) * h
        };

        let beckmann_pdf = |h: &glm::DVec3| {
            // p = 1 / (πm^2 cos^3 θ) * e^(-tan^2(θ) / m^2)
            let cos_t = h.dot(n).abs();
            let sin_t = (1.0 - cos_t * cos_t).sqrt();
            (std::f64::consts::PI * m2 * cos_t.powi(3)).recip()
                * (-(sin_t / cos_t).powi(2) / m2).exp()
        };

        let wi = if rng.gen_bool(f) {
            // Specular component
            let h = beckmann(rng);
            -glm::reflect_vec(wo, &h)
        } else if !self.transparent {
            // Diffuse component (Lambertian)
            // Simple cosine-sampling using Malley's method
            let [x, y]: [f64; 2] = rng.sample(UnitDisc);
            let z = (1.0_f64 - x * x - y * y).sqrt();
            local_to_world(n) * glm::vec3(x, y, z)
        } else {
            // Transmitted component
            let h = beckmann(rng);
            let cos_to = h.dot(wo);
            let wo_perp = wo - h * cos_to;
            let wi_perp = -wo_perp / eta_t;
            let sin2_ti = wi_perp.magnitude_squared();
            if sin2_ti > 1.0 {
                // This angle doesn't yield any transmittence to wo,
                // due to total internal reflection
                return None;
            }
            let cos_ti = (1.0 - sin2_ti).sqrt();
            -cos_to.signum() * cos_ti * h + wi_perp
        };

        // Multiple importance sampling - add up total probability
        let mut p = 0.0;
        p += {
            // Specular component
            let h = (wi + wo).normalize();
            let p_h = beckmann_pdf(&h);
            f * p_h / (4.0 * h.dot(wo).abs())
        };
        p += if !self.transparent {
            // Diffuse component
            (1.0 - f) * wi.dot(n).max(0.0) * std::f64::consts::FRAC_1_PI
        } else if wo.dot(n).is_sign_positive() != wi.dot(n).is_sign_positive() {
            // Transmitted component
            let h = (wi * eta_t + wo).normalize();
            let p_h = beckmann_pdf(&h);
            let h_dot_wo = h.dot(wo);
            let h_dot_wi = h.dot(&wi);
            let jacobian = h_dot_wo.abs() / (eta_t * h_dot_wi + h_dot_wo).powi(2);
            (1.0 - f) * p_h * jacobian
        } else {
            0.0
        };
        Some((wi, p))
    }
}

fn local_to_world(n: &glm::DVec3) -> glm::DMat3 {
    let ns = if n.x.is_normal() {
        glm::vec3(n.y, -n.x, 0.0).normalize()
    } else {
        glm::vec3(0.0, -n.z, n.y).normalize()
    };
    let nss = n.cross(&ns);
    glm::mat3(ns.x, nss.x, n.x, ns.y, nss.y, n.y, ns.z, nss.z, n.z)
}
