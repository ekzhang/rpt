use image::{ImageBuffer, RgbImage};

use crate::color::{color_bytes, Color};

/// A buffer that stores sample results from path tracing
pub struct Buffer {
    width:   u32,
    height:  u32,
    samples: Vec<Vec<Color>>,
    filter:  Filter,
}

impl Buffer {
    /// Construct a new buffer with a given width and height
    pub fn new(width: u32, height: u32, filter: Filter) -> Self {
        Self {
            width,
            height,
            samples: vec![vec![]; (width * height) as usize],
            filter,
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

    /// Return the average color variance of samples in each pixel
    pub fn variance(&self) -> f64 {
        let mut variance = 0.0;
        let mut count = 0.0;
        for pix_samples in &self.samples {
            let mean: Color = pix_samples.iter().sum::<Color>() / (pix_samples.len() as f64);
            let mut sum_of_squares = 0.0;
            for sample in pix_samples {
                sum_of_squares += (sample - mean).magnitude_squared();
            }
            // Sample variance: n - 1 degrees of freedom
            variance += sum_of_squares / (pix_samples.len() as f64 - 1.0);
            count += 1.0;
        }
        variance / count
    }

    fn get_filtered_color(&self, x: u32, y: u32) -> Color {
        match self.filter {
            Filter::Box(radius) => {
                let mut color = glm::vec3(0.0, 0.0, 0.0);
                let mut count = 0;
                for i in x.saturating_sub(radius)..=(x + radius) {
                    for j in y.saturating_sub(radius)..=(y + radius) {
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
    }
}

/// A noise reduction filter applied to the rendered image
#[derive(Copy, Clone)]
pub enum Filter {
    /// Box filter with a given radius
    Box(u32),
}

impl Default for Filter {
    fn default() -> Self {
        // Box filter with zero radius, which is a no-op
        Self::Box(0)
    }
}
