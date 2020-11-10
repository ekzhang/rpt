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
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.get_filtered_color(x, y);
                let [r, g, b] = color_bytes(&color);
                buf.push(r);
                buf.push(g);
                buf.push(b);
            }
        }
        ImageBuffer::from_raw(self.width, self.height, buf)
            .expect("Image buffer has incorrect size")
    }

    fn get_filtered_color(&self, x: u32, y: u32) -> Color {
        let mut color = glm::vec3(0.0, 0.0, 0.0);
        let mut count = 0;
        for i in x.saturating_sub(1)..=(x + 1) {
            for j in y.saturating_sub(1)..=(y + 1) {
                if i < self.width && j < self.height {
                    let index = (j * self.width + i) as usize;
                    color += self.samples[index].iter().sum::<Color>();
                    count += self.samples[index].len();
                }
            }
        }
        assert!(count != 0, "Pixel found with no samples");
        color / (count as f64)
    }
}
