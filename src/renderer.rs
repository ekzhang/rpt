use image::RgbImage;
use kd_tree::{ItemAndDistance, KdPoint, KdTree};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

use crate::buffer::{Buffer, Filter};
use crate::camera::Camera;
use crate::color::Color;
use crate::light::Light;
use crate::material::Material;
use crate::object::Object;
use crate::scene::Scene;
use crate::shape::{HitRecord, Ray};

const EPSILON: f64 = 1e-12;
const FIREFLY_CLAMP: f64 = 100.0;

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

    /// Exposure value (EV)
    pub exposure_value: f64,

    /// Optional noise-reduction filter
    pub filter: Filter,

    /// The maximum number of ray bounces
    pub max_bounces: u32,

    /// Number of random paths traced per pixel
    pub num_samples: u32,
}

impl<'a> Renderer<'a> {
    /// Construct a new renderer for a scene
    pub fn new(scene: &'a Scene, camera: Camera) -> Self {
        Self {
            scene,
            camera,
            width: 800,
            height: 600,
            exposure_value: 0.0,
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

    /// Set the exposure value of the rendered scene
    pub fn exposure_value(mut self, exposure_value: f64) -> Self {
        self.exposure_value = exposure_value;
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
                let mut rng = StdRng::from_entropy();
                (0..self.width)
                    .into_iter()
                    .map(|x| self.get_color(x, y, iterations, &mut rng))
                    .collect::<Vec<_>>()
            })
            .collect();
        buffer.add_samples(&colors);
    }

    fn get_color(&self, x: u32, y: u32, iterations: u32, rng: &mut StdRng) -> Color {
        let dim = std::cmp::max(self.width, self.height) as f64;
        let xn = ((2 * x + 1) as f64 - self.width as f64) / dim;
        let yn = ((2 * (self.height - y) - 1) as f64 - self.height as f64) / dim;
        let mut color = glm::vec3(0.0, 0.0, 0.0);
        for _ in 0..iterations {
            let dx = rng.gen_range((-1.0 / dim)..(1.0 / dim));
            let dy = rng.gen_range((-1.0 / dim)..(1.0 / dim));
            color += self.trace_ray(self.camera.cast_ray(xn + dx, yn + dy, rng), 0, rng);
        }
        color / f64::from(iterations) * 2.0_f64.powf(self.exposure_value)
    }

    /// Trace a ray, obtaining a Monte Carlo estimate of the luminance
    fn trace_ray(&self, ray: Ray, num_bounces: u32, rng: &mut StdRng) -> Color {
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
                            dir:    wi,
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
        rng: &mut StdRng,
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
                        dir:    wi,
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

struct Photon {
    pub position:  glm::DVec3,
    pub direction: glm::DVec3,
    pub power:     glm::DVec3,
}

impl KdPoint for Photon {
    type Scalar = f64;
    type Dim = typenum::U3;
    fn at(&self, k: usize) -> f64 {
        self.position[k]
    }
}

/// how many photons to count the contribution of when calculating indirect lighting for a point
static CLOSEST_N_PHOTONS: usize = 10;

impl<'a> Renderer<'a> {
    /// renders an image using photon mapping
    pub fn photon_map_render(&self, photon_count: usize) -> RgbImage {
        // ensure that scene only has object lights (may not be necessary)
        for light in self.scene.lights.iter() {
            match light {
                Light::Object(_) => {}
                _ => {
                    panic!("Only object lights are supported for photon mapping");
                }
            }
        }

        println!("Shooting photons");
        let watts = 100.;
        let mut rng = StdRng::from_entropy();
        let mut photon_list = Vec::new();
        for _ in 0..photon_count {
            // shoot photon from a random light
            photon_list.extend(self.shoot_photon(watts as f64, &mut rng));
        }
        // scale photons down to distribute the wattage
        photon_list
            .iter_mut()
            .for_each(|p| p.power /= photon_count as f64);

        println!("Building kdtree");
        let photon_map = KdTree::build_by(photon_list, |a, b, k| {
            a.position[k].partial_cmp(&b.position[k]).unwrap()
        });

        println!("Tracing rays");
        let mut buffer = Buffer::new(self.width, self.height, self.filter);
        let count = std::sync::atomic::AtomicUsize::new(0);
        let colors: Vec<_> = (0..self.height)
            .into_par_iter()
            .flat_map(|y| {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                println!("Count: {}", count.load(std::sync::atomic::Ordering::SeqCst));

                let mut rng = StdRng::from_entropy();
                (0..self.width)
                    .into_iter()
                    .map(|x| {
                        self.get_color_with_photon_map(
                            x,
                            y,
                            self.num_samples,
                            &mut rng,
                            &photon_map,
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        buffer.add_samples(&colors);

        buffer.image()
    }

    /// shoot a photon from a random light with power `power` and return a list of
    /// photons that have gathered in the scene
    fn shoot_photon(&self, power: f64, rng: &mut StdRng) -> Vec<Photon> {
        // FIXME: sample random light based on area instead of choosing randomly
        let light_index: usize = rng.gen_range(0..self.scene.lights.len());
        let light = &self.scene.lights[light_index as usize];

        // sample a random point on the light and a random direction in the hemisphere
        if let Light::Object(object) = light {
            // the `target` arg isn't used when sampling a triangle, so it can be a dummy value
            // Sample a location on the light
            let target = glm::vec3(0., 0., 0.);
            let (pos, n, pdf) = object.shape.sample(&target, rng);

            // sample random hemisphere direction
            let phi = 2. * glm::pi::<f64>() * rng.gen::<f64>();
            let theta = (1. - rng.gen::<f64>()).acos();
            let pdf_of_sample = 0.5 * glm::one_over_pi::<f64>();
            let random_hemisphere_dir = glm::vec3(
                theta.sin() * phi.sin(),
                theta.sin(),
                theta.sin() * phi.sin(),
            );

            // rotate direction towards normal
            let rotation = glm::quat_rotation(&glm::vec3(0., 1., 0.), &n);
            let direction = glm::quat_rotate_vec3(&rotation, &random_hemisphere_dir).normalize();

            // recursively gather photons
            let photons = self.trace_photon(
                Ray {
                    origin: pos,
                    dir:    direction,
                },
                power * object.material.color / pdf / pdf_of_sample,
                rng,
            );

            photons
        } else {
            panic!("Found non-object light while photon mapping")
        }
    }

    /// trace a photon along ray `ray` with power `power` and check for intersections. Returns
    /// vec of photons that have been recursively traces and placed in the scene
    fn trace_photon(&self, ray: Ray, power: glm::DVec3, rng: &mut StdRng) -> Vec<Photon> {
        match self.get_closest_hit(ray) {
            None => {
                // no photons if we don't hit a scene element
                Vec::new()
            }
            Some((h, object)) => {
                let world_pos = ray.at(h.time);
                let material = object.material;
                let wo = -glm::normalize(&ray.dir);

                // page 16 of siggraph course on photon mapping
                let specular = 1. - material.roughness;
                let specular = glm::vec3(specular, specular, specular);
                let diffuse = material.color;
                let specular = glm::vec3(0.1, 0.1, 0.1);
                let diffuse = glm::vec3(0.5, 0.5, 0.5);
                let p_r = vec![
                    specular.x + diffuse.x,
                    specular.y + diffuse.y,
                    specular.z + diffuse.z,
                ]
                .into_iter()
                .fold(f64::NAN, f64::max);
                let diffuse_sum = diffuse.x + diffuse.y + diffuse.z;
                let specular_sum = specular.x + specular.y + specular.z;
                let p_d = diffuse_sum / (diffuse_sum + specular_sum) * p_r;
                let p_s = specular_sum / (diffuse_sum + specular_sum) * p_r;

                // only do diffuse russian rouletter for now (no specular)
                let russian_roulette: f64 = rng.gen();
                if russian_roulette < p_d {
                    // diffuse reflection
                    if let Some((wi, pdf)) = material.sample_f(&h.normal, &wo, rng) {
                        let f = material.bsdf(&h.normal, &wo, &wi);
                        let ray = Ray {
                            origin: world_pos,
                            dir:    wi,
                        };

                        // account for the chance of terminating
                        let russian_roulette_scale_factor =
                            glm::vec3(diffuse.x / p_d, diffuse.y / p_d, diffuse.z / p_d);

                        // gather recursive photons with scaled down power
                        let mut next_photons: Vec<Photon> = self.trace_photon(
                            ray,
                            power
                                .component_mul(&russian_roulette_scale_factor)
                                .component_mul(&f)
                                * wi.dot(&h.normal)
                                / pdf,
                            rng,
                        );

                        // add photon from current step
                        next_photons.push(Photon {
                            position: world_pos,
                            direction: wo,
                            power,
                        });

                        next_photons
                    } else {
                        // total internal reflection: no photons
                        Vec::new()
                    }
                } else {
                    // absorbed
                    Vec::new()
                }
            }
        }
    }

    /// traces rays for a given pixel in the image and uses photon map to calc
    /// indirect lighting
    fn get_color_with_photon_map(
        &self,
        x: u32,
        y: u32,
        iterations: u32,
        rng: &mut StdRng,
        photon_map: &KdTree<Photon>,
    ) -> Color {
        let dim = std::cmp::max(self.width, self.height) as f64;
        let xn = ((2 * x + 1) as f64 - self.width as f64) / dim;
        let yn = ((2 * (self.height - y) - 1) as f64 - self.height as f64) / dim;
        let mut color = glm::vec3(0.0, 0.0, 0.0);
        for _ in 0..iterations {
            let dx = rng.gen_range((-1.0 / dim)..(1.0 / dim));
            let dy = rng.gen_range((-1.0 / dim)..(1.0 / dim));
            // trace ray
            color += self.trace_ray_with_photon_map(
                self.camera.cast_ray(xn + dx, yn + dy, rng),
                rng,
                photon_map,
            );
        }
        color / f64::from(iterations) * 2.0_f64.powf(self.exposure_value)
    }

    /// trace ray `Ray` through the scene to calculate illumination. Uses photon map
    /// for indirect illumination
    fn trace_ray_with_photon_map(
        &self,
        ray: Ray,
        rng: &mut StdRng,
        photon_map: &KdTree<Photon>,
    ) -> Color {
        match self.get_closest_hit(ray) {
            None => self.scene.environment.get_color(&ray.dir),
            Some((h, object)) => {
                let world_pos = ray.at(h.time);
                let material = object.material;
                let wo = -glm::normalize(&ray.dir);

                let near_photons = photon_map
                    .nearests(&[world_pos.x, world_pos.y, world_pos.z], CLOSEST_N_PHOTONS);

                // of the nearest photons, how far away is the furthest one?
                let max_dist_squared = near_photons
                    .iter()
                    .map(
                        |ItemAndDistance {
                             squared_distance, ..
                         }| { squared_distance },
                    )
                    .fold(0., |acc: f64, &p: &f64| acc.max(p));

                let mut color = Color::new(0.0, 0.0, 0.0);

                // indirect lighting via photon map
                for ItemAndDistance {
                    item: photon,
                    squared_distance: _,
                } in near_photons
                {
                    color += material
                        .bsdf(&h.normal, &wo, &photon.direction)
                        .component_mul(&photon.power);
                }

                // normalize by (1/(pi * r^2))
                color = color * (1. / (glm::pi::<f64>() * max_dist_squared));

                // direct lighting via light sampling
                color += self.sample_lights(&material, &world_pos, &h.normal, &wo, rng);

                // emitted lighting
                color += material.emittance * material.color;

                color
            }
        }
    }
}
