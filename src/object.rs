use std::rc::Rc;

use crate::material::Material;
use crate::shape::Shape;

/// An object rendered in a scene
pub struct Object {
    /// Basic geometry of the object
    pub shape: Rc<dyn Shape>,

    /// Material of the object (possibly simple or complex)
    pub material: Material,

    /// Affine transform applied to the object
    pub transform: glm::Mat4,
}

impl Object {
    /// Create a new object from a shape, with default material
    pub fn new(shape: Rc<dyn Shape>) -> Self {
        Self {
            shape,
            material: Material::default(),
            transform: glm::identity(),
        }
    }

    /// Set the material of the object (builder pattern)
    pub fn material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    /// Transform: prepend a translation
    pub fn translate(mut self, v: &glm::Vec3) -> Self {
        self.transform = glm::translate(&self.transform, v);
        self
    }

    /// Transform: prepend a scale, in 3 dimensions
    pub fn scale(mut self, v: &glm::Vec3) -> Self {
        self.transform = glm::scale(&self.transform, v);
        self
    }

    /// Transform: prepend a rotation, by an angle in radians about an axis
    pub fn rotate(mut self, angle: f32, axis: &glm::Vec3) -> Self {
        self.transform = glm::rotate(&self.transform, angle, axis);
        self
    }

    /// Transform: prepend a rotation around the X axis, by an angle in radians
    pub fn rotate_x(mut self, angle: f32) -> Self {
        self.transform = glm::rotate_x(&self.transform, angle);
        self
    }

    /// Transform: prepend a rotation around the Y axis, by an angle in radians
    pub fn rotate_y(mut self, angle: f32) -> Self {
        self.transform = glm::rotate_y(&self.transform, angle);
        self
    }

    /// Transform: prepend a rotation around the Z axis, by an angle in radians
    pub fn rotate_z(mut self, angle: f32) -> Self {
        self.transform = glm::rotate_z(&self.transform, angle);
        self
    }
}
