use crate::color::{color_bytes, Color};
use crate::scene::Scene;
use image::{ImageBuffer, RgbImage};

/// Builder object for rendering a scene
pub struct Renderer<'a> {
    /// The scene to be rendered
    pub scene: &'a Scene,

    /// The camera to use
    pub camera: Camera,

    /// The width of the output image
    pub width: u32,

    /// The height of the output image
    pub height: u32,
}

/// A simple perspective camera
#[derive(Copy, Clone)]
pub struct Camera {
    /// Location of the camera
    pub center: glm::Vec3,

    /// Direction that the camera is facing
    pub direction: glm::Vec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: glm::Vec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            fov: std::f32::consts::FRAC_PI_6,
        }
    }
}

impl<'a> Renderer<'a> {
    /// Construct a new renderer for a scene
    pub fn new(scene: &'a Scene, camera: Camera) -> Self {
        Self {
            scene,
            camera,
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
