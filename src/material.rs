use crate::color::{hex_color, Color};

/// Represents a shader material with some physical properties
///
/// TODO: more advanced materials, physically accurate (Fresnel)
pub struct Material {
    /// Color of the material
    pub diffuse: Color,

    /// Specular coefficient
    pub specular: Color,

    /// Shininess (specular angle power-law falloff)
    pub shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            diffuse: hex_color(0xff0000), // red
            specular: glm::vec3(0.9, 0.9, 0.9),
            shininess: 20.0,
        }
    }
}
