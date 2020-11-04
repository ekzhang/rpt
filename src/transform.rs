/// Represents an affine transformation
#[derive(Copy, Clone)]
pub struct Transform {
    /// The projective transformation matrix
    pub matrix: glm::Mat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            matrix: glm::identity(),
        }
    }
}
