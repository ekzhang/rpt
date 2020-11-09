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
            color: glm::vec3(0.0, 0.0, 0.0),
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
