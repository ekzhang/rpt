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

impl Transform {
    /// Construct a new identity transformation
    pub fn new() -> Self {
        Default::default()
    }

    /// Prepend a translation
    pub fn translate(mut self, v: &glm::Vec3) -> Self {
        self.matrix = glm::translate(&self.matrix, v);
        self
    }

    /// Prepend a scale, in 3 dimensions
    pub fn scale(mut self, v: &glm::Vec3) -> Self {
        self.matrix = glm::scale(&self.matrix, v);
        self
    }

    /// Prepend a rotation, expressed as an angle in radians about an axis
    pub fn rotate(mut self, angle: f32, axis: &glm::Vec3) -> Self {
        self.matrix = glm::rotate(&self.matrix, angle, axis);
        self
    }

    /// Prepend a rotation around the X axis, by an angle in radians
    pub fn rotate_x(mut self, angle: f32) -> Self {
        self.matrix = glm::rotate_x(&self.matrix, angle);
        self
    }

    /// Prepend a rotation around the Y axis, by an angle in radians
    pub fn rotate_y(mut self, angle: f32) -> Self {
        self.matrix = glm::rotate_y(&self.matrix, angle);
        self
    }

    /// Prepend a rotation around the Z axis, by an angle in radians
    pub fn rotate_z(mut self, angle: f32) -> Self {
        self.matrix = glm::rotate_z(&self.matrix, angle);
        self
    }
}
