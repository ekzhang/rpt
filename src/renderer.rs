use crate::color::{color_bytes, Color};
use crate::scene::Scene;
use image::{ImageBuffer, RgbImage};

/// Builder object for rendering a scene
pub struct Renderer<'a> {
    /// The scene to be rendered
    scene: &'a Scene,

    /// The width of the output image
    width: u32,

    /// The height of the output image
    height: u32,
}

impl<'a> Renderer<'a> {
    /// Construct a new renderer for a scene
    pub fn new(scene: &'a Scene) -> Self {
        Self {
            scene,
            width: 800,
            height: 600,
        }
    }

    /// Set the width of the rendered scene
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Set the height of the rendered scene
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Render the scene by path tracing
    pub fn render(self) -> RgbImage {
        ImageBuffer::from_fn(self.width, self.height, |x, y| {
            image::Rgb(color_bytes(&self.get_color(x, y)))
        })
    }

    fn get_color(&self, _x: u32, _y: u32) -> Color {
        self.scene.background
    }
}
