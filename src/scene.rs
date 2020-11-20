use crate::environment::Environment;
use crate::light::Light;
use crate::object::Object;

/// Object representing a scene that can be rendered
#[derive(Default)]
pub struct Scene {
    /// Collection of objects in the scene
    pub objects: Vec<Object>,

    /// Collection of lights in the scene
    pub lights: Vec<Light>,

    /// Environment map used for scene lighting
    pub environment: Environment,
}

impl Scene {
    /// Construct a new, empty scene
    pub fn new() -> Self {
        Default::default()
    }
}

/// Trait that allows adding an object or light to a scene
pub trait SceneAdd<T> {
    /// Add an object or light to the scene
    fn add(&mut self, node: T);
}

impl SceneAdd<Object> for Scene {
    fn add(&mut self, object: Object) {
        self.objects.push(object);
    }
}

impl SceneAdd<Light> for Scene {
    fn add(&mut self, light: Light) {
        self.lights.push(light);
    }
}
