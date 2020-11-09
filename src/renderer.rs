use image::{ImageBuffer, RgbImage};
use rayon::prelude::*;

use crate::color::{color_bytes, Color};
use crate::light::Light;
use crate::object::Object;
use crate::scene::Scene;
use crate::shape::{HitRecord, Ray};

const EPSILON: f64 = 1e-12;

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
    pub center: glm::DVec3,

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
            center: glm::vec3(0.0, 0.0, 10.0),
            direction: glm::vec3(0.0, 0.0, -1.0),
            up: glm::vec3(0.0, 1.0, 0.0), // we live in a y-up world...
            fov: std::f64::consts::FRAC_PI_6,
        }
    }
}

impl Camera {
    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    pub fn cast_ray(&self, x: f64, y: f64) -> Ray {
        // cot(f / 2) = depth / radius
        let d = (self.fov / 2.0).tan().recip();
        let right = glm::cross(&self.direction, &self.up).normalize();
        let new_dir = d * self.direction + x * right + y * self.up;
        Ray {
            origin: self.center,
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
    pub fn render(&self) -> RgbImage {
        let buf: Vec<_> = (0..self.height)
            .into_par_iter()
            .flat_map(|y| {
                let mut buf = Vec::new();
                for x in 0..self.width {
                    let [r, g, b] = color_bytes(&self.get_color(x, y));
                    buf.push(r);
                    buf.push(g);
                    buf.push(b);
                }
                buf
            })
            .collect();
        ImageBuffer::from_raw(self.width, self.height, buf).expect("Image buffer has wrong size")
    }

    fn get_color(&self, x: u32, y: u32) -> Color {
        let dim = std::cmp::max(self.width, self.height) as f64;
        let xn = ((2 * x + 1) as f64 - self.width as f64) / dim;
        let yn = ((2 * (self.height - y) - 1) as f64 - self.height as f64) / dim;
        self.trace_ray(self.camera.cast_ray(xn, yn), 0)
    }

    fn trace_ray(&self, ray: Ray, num_bounces: u32) -> Color {
        if num_bounces > 0 {
            todo!();
        }
        match self.get_closest_hit(ray) {
            None => self.scene.background,
            Some((h, object)) => {
                let world_pos = ray.at(h.time);
                let eye = -glm::normalize(&ray.dir);
                let mat = object.material;
                let mut color = mat.emittance * mat.color;

                for light in &self.scene.lights {
                    if let Light::Ambient(ambient_color) = light {
                        color += ambient_color.component_mul(&object.material.color);
                    } else {
                        let (intensity, dir_to_light, dist_to_light) = light.illuminate(world_pos);
                        let closest_hit = self
                            .get_closest_hit(Ray {
                                origin: world_pos,
                                dir: dir_to_light,
                            })
                            .map(|(r, _)| r.time);

                        if closest_hit.is_none() || closest_hit.unwrap() > dist_to_light {
                            // Cook-Torrance BRDF with GGX microfacet distribution
                            let f = mat.bsdf(&h.normal, &eye, &dir_to_light);
                            color += f.component_mul(&intensity) * dir_to_light.dot(&h.normal);
                        }
                    }
                }

                color
            }
        }
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
