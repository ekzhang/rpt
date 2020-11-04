use crate::color::Color;

/// Object representing a scene that can be rendered
#[derive(Default)]
pub struct Scene {
    /*
    pub shapes: Vec<Shape>,
    pub lights: Vec<Light>,
    */
    /// The color used for background pixels
    pub background: Color,
}

impl Scene {
    /// Construct a new, empty scene
    pub fn new() -> Self {
        Default::default()
    }
}
