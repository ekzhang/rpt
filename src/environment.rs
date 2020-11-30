use crate::color::Color;

/// High-dynamic-range equirectangular image for lighting 3D scenes
#[derive(Clone)]
pub struct Hdri {
    /// Width of the image
    width: u32,

    /// Height of the image
    height: u32,

    /// Buffer of floating-point RGB pixels
    buf: Vec<Color>,
}

impl Hdri {
    /// Create a new HDRI image
    pub fn new(width: u32, height: u32, buf: Vec<Color>) -> Self {
        assert!(buf.len() == width as usize * height as usize);
        assert!(width > 0 && height > 0);
        Self { width, height, buf }
    }

    /// Sample a color from a direction in the environment
    pub fn get_color(&self, dir: &glm::DVec3) -> Color {
        let dir = dir.normalize();
        let azimuth = dir.z.atan2(dir.x) + std::f64::consts::PI;
        let polar = dir.y.acos();
        let x = azimuth / std::f64::consts::TAU * (self.width - 1) as f64;
        let y = polar / std::f64::consts::PI * (self.height - 1) as f64;
        self.bilinear_sample(x, y)
    }

    fn bilinear_sample(&self, x: f64, y: f64) -> Color {
        let x0 = (x as u32).min(self.width - 1);
        let y0 = (y as u32).min(self.height - 1);
        let ax = x - x0 as f64;
        let ay = y - y0 as f64;
        glm::mix(
            &glm::mix(
                &self.buf[(y0 * self.width + x0) as usize],
                &self.buf[(y0 * self.width + x0 + 1) as usize],
                ax,
            ),
            &glm::mix(
                &self.buf[((y0 + 1) * self.width + x0) as usize],
                &self.buf[((y0 + 1) * self.width + x0 + 1) as usize],
                ax,
            ),
            ay,
        )
    }
}

/// An environment map for lighting 3D scenes
pub enum Environment {
    /// Solid-color environment lighting
    Color(Color),

    /// High-dynamic-range image environment lighting
    Hdri(Hdri),
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Color(glm::vec3(0.0, 0.0, 0.0))
    }
}

impl Environment {
    /// Sample a color from a direction in the environment
    pub fn get_color(&self, dir: &glm::DVec3) -> Color {
        match self {
            Self::Color(color) => *color,
            Self::Hdri(hdri) => hdri.get_color(dir),
        }
    }
}
