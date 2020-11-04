/// Type representing various forms of lighting
pub enum Light {
    /// Point light represented as (color, location)
    Point(glm::Vec3, glm::Vec3),

    /// Ambient light represented as (color)
    Ambient(glm::Vec3),

    /// Directional light represented as (color, direction)
    Directional(glm::Vec3, glm::Vec3),
}

impl Light {
    /// Illuminates a point, returning (dir_to_light, intensity, dist_to_light)
    pub fn illuminate(&self, world_pos: glm::Vec3) -> (glm::Vec3, glm::Vec3, f32) {
        match self {
            Light::Ambient(color) => (glm::vec3(0.0, 0.0, 0.0), *color, 0.0),
            Light::Point(color, location) => {
                let disp = location - world_pos;
                let len = glm::length(&disp);
                (disp / len, color / (len * len), len)
            }
            Light::Directional(color, direction) => {
                (-glm::normalize(direction), *color, f32::INFINITY)
            }
        }
    }
}
