//! `rpt` is a path tracer in Rust.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use image::{ImageBuffer, RgbImage};

/// A representation of an RGB color
pub type Color = glm::Vec3;

/// Construct a new color from a hex string, such as `hex_color(0xab23f0)`
pub fn hex_color(x: u32) -> Color {
    let r = ((x >> 16) & 0xff) as f32 / 255.0;
    let g = ((x >> 8) & 0xff) as f32 / 255.0;
    let b = ((x >> 0) & 0xff) as f32 / 255.0;
    glm::vec3(r, g, b)
}

/// Convert a color to a clamped triple of unsigned bytes
pub fn color_bytes(color: &Color) -> [u8; 3] {
    [
        (color.x.max(0.0).min(1.0) * 255.0) as u8,
        (color.y.max(0.0).min(1.0) * 255.0) as u8,
        (color.z.max(0.0).min(1.0) * 255.0) as u8,
    ]
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_work() {
        let black = hex_color(0x000000);
        let white = hex_color(0xffffff);
        let red = hex_color(0xff0000);
        assert_eq!(color_bytes(&black), [0, 0, 0]);
        assert_eq!(color_bytes(&white), [255, 255, 255]);
        assert_eq!(color_bytes(&red), [255, 0, 0]);
    }
}
