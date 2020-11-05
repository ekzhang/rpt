use crate::material::Material;
use crate::shape::Shape;
use crate::transform::Transform;
use std::rc::Rc;

/// An object rendered in a scene
pub struct Object {
    /// Basic geometry of the object
    pub shape: Rc<dyn Shape>,

    /// Material of the object (possibly simple or complex)
    pub material: Material,

    /// Affine transform applied to the object
    pub transform: Transform,
}

impl Object {
    /// Create a new object from a shape, with default material
    pub fn new(shape: Rc<dyn Shape>) -> Self {
        Self {
            shape,
            material: Material::default(),
            transform: Transform::default(),
        }
    }

    /// Set the material of the object (builder pattern)
    pub fn material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    /// Set the transform of the object (builder pattern)
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
}
