use crate::color::{color_bytes, Color};
use crate::light::Light;
use crate::object::Object;
use crate::scene::Scene;
use crate::shape::{HitRecord, Ray};
use image::{ImageBuffer, RgbImage};

const EPSILON: f32 = 1e-4;

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

impl Camera {
    /// Cast a ray, where (x, y) are normalized to the standard [-1, 1] box
    pub fn cast_ray(&self, x: f32, y: f32) -> Ray {
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
        // TODO: parallelize this code (should be easy)
        ImageBuffer::from_fn(self.width, self.height, |x, y| {
            image::Rgb(color_bytes(&self.get_color(x, y)))
        })
    }

    fn get_color(&self, x: u32, y: u32) -> Color {
        let dim = std::cmp::max(self.width, self.height) as f32;
        let xn = ((2 * x + 1) as f32 - self.width as f32) / dim;
        let yn = ((2 * (self.height - y) - 1) as f32 - self.height as f32) / dim;
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
                let eye_r = glm::reflect_vec(&glm::normalize(&ray.dir), &h.normal);
                let mut color = glm::vec3(0.0, 0.0, 0.0);

                for light in &self.scene.lights {
                    if let Light::Ambient(ambient_color) = light {
                        color += ambient_color;
                    } else {
                        let (intensity, dir_to_light, dist_to_light) = light.illuminate(world_pos);
                        let closest_hit = self
                            .get_closest_hit(Ray {
                                origin: world_pos,
                                dir: dir_to_light,
                            })
                            .map(|(r, _)| r.time)
                            .unwrap_or(f32::INFINITY);

                        if closest_hit > dist_to_light {
                            // Phong reflectance model (BRDF)
                            let kd = f32::max(0.0, glm::dot(&dir_to_light, &h.normal));
                            let diffuse = kd * intensity.component_mul(&object.material.diffuse);
                            let ks = f32::max(0.0, glm::dot(&dir_to_light, &eye_r))
                                .powf(object.material.shininess);
                            let specular = ks * intensity.component_mul(&object.material.specular);
                            color += diffuse + specular;
                        }
                    }
                }

                color
            }
        }
    }

    fn get_closest_hit(&self, ray: Ray) -> Option<(HitRecord, &'_ Object)> {
        let mut h = HitRecord::new();
        let mut hit = None;
        for object in &self.scene.objects {
            let local_ray = ray.apply_transform(&glm::inverse(&object.transform));
            if object.shape.intersect(&local_ray, EPSILON, &mut h) {
                hit = Some(object);
                // Fix normal vectors by multiplying by M^-T
                h.normal = (glm::inverse_transpose(glm::mat4_to_mat3(&object.transform))
                    * h.normal)
                    .normalize();
            }
        }
        Some((h, hit?))
    }
}
