use image::{ImageBuffer, RgbImage};

use crate::color::{color_bytes, Color};

/// A buffer that stores sample results from path tracing
pub struct Buffer {
    width: u32,
    height: u32,
    samples: Vec<Vec<Color>>,
}

impl Buffer {
    /// Construct a new buffer with a given width and height
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            samples: vec![vec![]; (width * height) as usize],
        }
    }

    /// Add a sample to the buffer, at a given pixel location
    pub fn add_sample(&mut self, x: u32, y: u32, sample: Color) {
        assert!(x < self.width && y < self.height, "Invalid pixel location");
        let index = (y * self.width + x) as usize;
        self.samples[index].push(sample);
    }

    /// Add a uniform matrix of samples to the buffer
    pub fn add_samples(&mut self, samples: &[Color]) {
        assert!(
            samples.len() == (self.width * self.height) as usize,
            "Invalid sample dimension"
        );
        for (index, sample) in samples.iter().enumerate() {
            self.samples[index].push(*sample);
        }
    }

    /// Converts the current buffer to an image
    pub fn image(&self) -> RgbImage {
        let mut buf = Vec::new();
        for pix_samples in &self.samples {
            assert!(
                !pix_samples.is_empty(),
                "Pixel found with no samples, cannot generate image."
            );
            let color = pix_samples.iter().sum::<glm::DVec3>() / (pix_samples.len() as f64);
            let [r, g, b] = color_bytes(&color);
            buf.push(r);
            buf.push(g);
            buf.push(b);
        }
        ImageBuffer::from_raw(self.width, self.height, buf)
            .expect("Image buffer has incorrect size")
    }
}
