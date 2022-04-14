use crate::material::Material;
use crate::shape::Shape;

/// An object rendered in a scene
///
/// In the future, it may be more flexible to have an object trait that alllows a user
/// to get the material at an intersection point. This would make sense to help for
/// something like kd-trees, as it would let you create a kd-tree of different materials,
/// and it would also work well with texture mapping.
pub struct Object {
    /// Basic geometry of the object
    pub shape: Box<dyn Shape>,

    /// Material of the object (possibly simple or complex)
    pub material: Material,
}

impl Object {
    /// Create a new object from a shape, with default material
    pub fn new<T: Shape + 'static>(shape: T) -> Self {
        Self {
            shape:    Box::new(shape),
            material: Material::default(),
        }
    }

    /// Set the material of the object (builder pattern)
    pub fn material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }
}
