use rand::rngs::StdRng;

use crate::color::Color;
use crate::object::Object;

/// Type representing various forms of lighting
pub enum Light {
    /// Point light represented as (color, location)
    Point(Color, glm::DVec3),

    /// Ambient light represented as (color)
    Ambient(Color),

    /// Directional light represented as (color, direction)
    Directional(Color, glm::DVec3),

    /// Light from an invisible, emissive object
    Object(Object),
}

impl Light {
    /// Illuminates a point, returning (intensity, dir_to_light, dist_to_light)
    pub fn illuminate(&self, world_pos: &glm::DVec3, rng: &mut StdRng) -> (Color, glm::DVec3, f64) {
        match self {
            Light::Ambient(color) => (*color, glm::vec3(0.0, 0.0, 0.0), 0.0),
            Light::Point(color, location) => {
                let disp = location - world_pos;
                let len = glm::length(&disp);
                (color / (len * len), disp / len, len)
            }
            Light::Directional(color, direction) => {
                (*color, -glm::normalize(direction), f64::INFINITY)
            }
            Light::Object(object) => {
                let (v, n, p) = object.shape.sample(&world_pos, rng);
                let disp = v - world_pos;
                let len = glm::length(&disp);
                let cosine = (-disp.dot(&n)).max(0.0) / len;
                let surface_area = cosine.max(0.0) / (len * len);
                (
                    object.material.color * object.material.emittance * surface_area / p,
                    disp / len,
                    len,
                )
            }
        }
    }
}
