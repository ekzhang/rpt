use crate::color::{hex_color, Color};

/// Represents a shader material with some physical properties
#[derive(Copy, Clone)]
/// This material shader is adapted from [https://github.com/fogleman/pt/].
pub struct Material {
    /// Albedo color
    pub color: Color,

    /// Index of refraction
    pub index: f64,

    /// Roughness parameter for GGX microfacet distribution
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

impl Material {
    /// Bidirectional scattering distribution function
    ///
    /// - `n` - surface normal vector
    /// - `wo` - unit direction vector toward the viewer
    /// - `wi` - unit direction vector toward the incident ray
    ///
    /// Right now, this only works for opaque materials. Implementing refraction
    /// and transmittence is TODO. Useful references:
    ///
    /// - http://www.codinglabs.net/article_physically_based_rendering_cook_torrance.aspx
    /// - https://computergraphics.stackexchange.com/q/4394
    /// - https://graphics.stanford.edu/courses/cs148-10-summer/docs/2006--degreve--reflection_refraction.pdf
    /// - http://www.pbr-book.org/3ed-2018/Materials/BSDFs.html
    pub fn bsdf(&self, n: &glm::DVec3, wo: &glm::DVec3, wi: &glm::DVec3) -> Color {
        let h = (wi + wo).normalize(); // halfway vector
        let nh2 = h.dot(n).powf(2.0);

        // d: microfacet distribution function
        // D = exp(((n • h)^2 - 1) / (m^2 (n • h)^2)) / (π m^2 (n • h)^4)
        let m2 = self.roughness * self.roughness;
        let d = ((nh2 - 1.0) / (m2 * nh2)).exp() / (m2 * glm::pi::<f64>() * nh2 * nh2);

        // f: fresnel, schlick's approximation
        // F = F0 + (1 - F0)(1 - wi • h)^5
        let f0 = ((self.index - 1.0) / (self.index + 1.0)).powf(2.0);
        let f0 = glm::lerp(&glm::vec3(f0, f0, f0), &self.color, self.metallic);
        let f = f0 + (glm::vec3(1.0, 1.0, 1.0) - f0) * (1.0 - wi.dot(&h)).powf(5.0);

        // g: geometry function, microfacet shadowing
        // G = min(1, 2(n • h)(n • wo)/(wo • h), 2(n • h)(n • wi)/(wo • h))
        let g = f64::min(n.dot(wi), n.dot(wo));
        let g = (2.0 * n.dot(&h) * g) / (wo.dot(&h));
        let g = g.min(1.0);

        // BRDF: putting it all together
        // Cook-Torrance = DFG / (4(n • wk)(n • wo))
        // Lambert = (1 - F) * c / π
        let specular = d * f * g / (4.0 * n.dot(wo) * n.dot(wi));
        let diffuse = (glm::vec3(1.0, 1.0, 1.0) - f).component_mul(&self.color) / glm::pi::<f64>();
        specular + diffuse
    }
}
