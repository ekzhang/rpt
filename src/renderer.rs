use image::RgbImage;
use rand::{rngs::ThreadRng, Rng};
use rayon::prelude::*;

use crate::buffer::{Buffer, Filter};
use crate::color::Color;
use crate::light::Light;
use crate::material::Material;
use crate::object::Object;
use crate::scene::Scene;
use crate::shape::{HitRecord, Ray};

const EPSILON: f64 = 1e-12;
const FIREFLY_CLAMP: f64 = 10.0;

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

    /// Optional noise-reduction filter
    pub filter: Filter,

    /// The maximum number of ray bounces
    pub max_bounces: u32,

    /// Number of random paths traced per pixel
    pub num_samples: u32,
}

/// A simple perspective camera
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// Location of the camera
    pub eye: glm::DVec3,

    /// Direction that the camera is facing
    pub direction: glm::DVec3,

    /// Direction of "up" for screen, must be orthogonal to `direction`
    pub up: glm::DVec3,

    /// Field of view in the longer direction as an angle in radians, in (0, pi)
    pub fov: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            fov: std::f64::consts::FRAC_PI_6,
        }
    }
}

impl Camera {
    /// Perspective camera looking at a point, with a given field of view
    pub fn look_at(eye: glm::DVec3, center: glm::DVec3, up: glm::DVec3, fov: f64) -> Self {
        let direction = (center - eye).normalize();
        let up = (up - up.dot(&direction) * direction).normalize();
        Self {
            eye,
            direction,
            up,
            fov,
        }
    }

    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    pub fn cast_ray(&self, x: f64, y: f64) -> Ray {
        // cot(f / 2) = depth / radius
        let d = (self.fov / 2.0).tan().recip();
        let right = glm::cross(&self.direction, &self.up).normalize();
        let new_dir = d * self.direction + x * right + y * self.up;
        Ray {
            origin: self.eye,
            dir: new_dir.normalize(),
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
            filter: Filter::default(),
            max_bounces: 0,
            num_samples: 1,
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

    /// Set the noise reduction filter
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    /// Set the maximum number of ray bounces when ray is traced
    pub fn max_bounces(mut self, max_bounces: u32) -> Self {
        self.max_bounces = max_bounces;
        self
    }

    /// Set the number of random paths traced per pixel
    pub fn num_samples(mut self, num_samples: u32) -> Self {
        self.num_samples = num_samples;
        self
    }

    /// Render the scene by path tracing
    pub fn render(&self) -> RgbImage {
        let mut buffer = Buffer::new(self.width, self.height, self.filter);
        self.sample(self.num_samples, &mut buffer);
        buffer.image()
    }

    /// Render the scene iteratively, calling a callback after every k samples
    pub fn iterative_render<F>(&self, callback_interval: u32, mut callback: F)
    where
        F: FnMut(u32, &Buffer),
    {
        let mut buffer = Buffer::new(self.width, self.height, self.filter);
        let mut iteration = 0;
        while iteration < self.num_samples {
            let steps = std::cmp::min(self.num_samples - iteration, callback_interval);
            self.sample(steps, &mut buffer);
            iteration += steps;
            callback(iteration, &buffer);
        }
    }

    fn sample(&self, iterations: u32, buffer: &mut Buffer) {
        let colors: Vec<_> = (0..self.height)
            .into_par_iter()
            .flat_map(|y| {
                (0..self.width)
                    .into_iter()
                    .map(|x| self.get_color(x, y, iterations))
                    .collect::<Vec<_>>()
            })
            .collect();
        buffer.add_samples(&colors);
    }

    fn get_color(&self, x: u32, y: u32, iterations: u32) -> Color {
        let dim = std::cmp::max(self.width, self.height) as f64;
        let xn = ((2 * x + 1) as f64 - self.width as f64) / dim;
        let yn = ((2 * (self.height - y) - 1) as f64 - self.height as f64) / dim;
        let mut rng = rand::thread_rng();
        let mut color = glm::vec3(0.0, 0.0, 0.0);
        for _ in 0..iterations {
            let dx = rng.gen_range(-1.0 / dim, 1.0 / dim);
            let dy = rng.gen_range(-1.0 / dim, 1.0 / dim);
            color += self.trace_ray(self.camera.cast_ray(xn + dx, yn + dy), 0, &mut rng);
        }
        color / f64::from(iterations)
    }

    /// Trace a ray, obtaining a Monte Carlo estimate of the luminance
    fn trace_ray(&self, ray: Ray, num_bounces: u32, rng: &mut ThreadRng) -> Color {
        match self.get_closest_hit(ray) {
            None => self.scene.environment.get_color(&ray.dir),
            Some((h, object)) => {
                let world_pos = ray.at(h.time);
                let material = object.material;
                let wo = -glm::normalize(&ray.dir);

                let mut color = material.emittance * material.color;
                color += self.sample_lights(&material, &world_pos, &h.normal, &wo, rng);
                if num_bounces < self.max_bounces {
                    if let Some((wi, pdf)) = material.sample_f(&h.normal, &wo, rng) {
                        let f = material.bsdf(&h.normal, &wo, &wi);
                        let ray = Ray {
                            origin: world_pos,
                            dir: wi,
                        };
                        let indirect = 1.0 / pdf
                            * f.component_mul(&self.trace_ray(ray, num_bounces + 1, rng))
                            * wi.dot(&h.normal).abs();
                        color.x += indirect.x.min(FIREFLY_CLAMP);
                        color.y += indirect.y.min(FIREFLY_CLAMP);
                        color.z += indirect.z.min(FIREFLY_CLAMP);
                    }
                }

                color
            }
        }
    }

    /// Explicitly sample from all the lights in the scene
    fn sample_lights(
        &self,
        material: &Material,
        pos: &glm::DVec3,
        n: &glm::DVec3,
        wo: &glm::DVec3,
        rng: &mut ThreadRng,
    ) -> Color {
        let mut color = glm::vec3(0.0, 0.0, 0.0);
        for light in &self.scene.lights {
            if let Light::Ambient(ambient_color) = light {
                color += ambient_color.component_mul(&material.color);
            } else {
                let (intensity, wi, dist_to_light) = light.illuminate(pos, rng);
                let closest_hit = self
                    .get_closest_hit(Ray {
                        origin: *pos,
                        dir: wi,
                    })
                    .map(|(r, _)| r.time);
                if closest_hit.is_none() || closest_hit.unwrap() > dist_to_light {
                    let f = material.bsdf(n, wo, &wi);
                    color += f.component_mul(&intensity) * wi.dot(n);
                }
            }
        }
        color
    }

    /// Loop through all objects in the scene to find the closest hit.
    ///
    /// Note that we intentionally do not use a `KdTree` to accelerate this computation.
    /// The reason is that some objects, like planes, have infinite extent, so it would
    /// not be appropriate to put them indiscriminately into a kd-tree.
    fn get_closest_hit(&self, ray: Ray) -> Option<(HitRecord, &'_ Object)> {
        let mut h = HitRecord::new();
        let mut hit = None;
        for object in &self.scene.objects {
            if object.shape.intersect(&ray, EPSILON, &mut h) {
                hit = Some(object);
            }
        }
        Some((h, hit?))
    }
}
