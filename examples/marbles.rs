use image::{
    codecs::hdr::{HdrDecoder, HdrMetadata},
    ImageResult, Rgb,
};
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::io::BufReader;
use std::process::Command;
use std::sync::Arc;

use rpt::*;

fn rgb_to_color(rgb: Rgb<f32>) -> Color {
    glm::vec3(rgb.0[0] as f64, rgb.0[1] as f64, rgb.0[2] as f64)
}

fn load_hdr(url: &str) -> ImageResult<Hdri> {
    let reader = ureq::get(url).call().into_reader();
    let decoder = HdrDecoder::new(BufReader::new(reader))?;
    let HdrMetadata { width, height, .. } = decoder.metadata();
    let pix = decoder.read_image_hdr()?;
    Ok(Hdri::new(
        width,
        height,
        pix.into_iter().map(rgb_to_color).collect(),
    ))
}

const TEST: bool = false;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    std::fs::create_dir_all("video")?;

    const N: usize = 25;
    let mut rng = rand::rngs::StdRng::seed_from_u64(123);

    let pos = (0..N)
        .map(|i| {
            glm::vec3(
                (i / 5) as f64 / 5. - 0.375,
                rng.gen_range(4., 6.),
                (i % 5) as f64 / 5. - 0.375,
            )
        })
        .collect();
    let mut cur_state = ParticleState {
        pos,
        vel: vec![glm::vec3(0., 0., 0.); N],
    };
    const R: f64 = 0.15;

    let system = MarblesSystem { radius: R };

    let hdri = load_hdr("https://hdrihaven.com/files/hdris/ballroom_8k.hdr")?;
    let surface_shape =
        Arc::new(load_obj(File::open("examples/monomial.obj")?)?.scale(&glm::vec3(1., 1., 1.)));
    for frame in 0..180 {
        let mut scene = Scene::new();
        if !TEST {
            scene.environment = Environment::Hdri(hdri.clone());
            scene.add(Light::Object(
                Object::new(
                    sphere()
                        .scale(&glm::vec3(1.5, 1.5, 1.5))
                        .translate(&glm::vec3(0.0, 5.0, 0.0)),
                )
                .material(Material::light(hex_color(0xFFFFFF), 15.0)),
            ));
        } else {
            scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
        }

        let glass = Material::clear(1.5, 0.0001);
        scene.add(Object::new(surface_shape.clone()).material(glass));
        let colors = [0x264653, 0x2A9D8F, 0xE9C46A, 0xF4A261, 0xE76F51];
        let surf = monomial_surface(2., 4.);
        for i in 0..N {
            let mut pos = cur_state.pos[i];
            let closest = surf.closest_point_precise(&pos);
            let vec = pos - closest;
            if glm::length(&vec) < R * 1.05 {
                pos = closest + glm::normalize(&vec) * R * 1.05;
            }
            pos.y = pos.y.max(R - 0.06);
            scene.add(
                Object::new(sphere().scale(&glm::vec3(R, R, R)).translate(&pos))
                    .material(Material::specular(hex_color(colors[i % colors.len()]), 0.1)),
            );
        }
        scene.add(
            Object::new(polygon(&[
                glm::vec3(20.0, -0.06, 20.0),
                glm::vec3(20.0, -0.06, -20.0),
                glm::vec3(-20.0, -0.06, -20.0),
                glm::vec3(-20.0, -0.06, 20.0),
            ]))
            .material(Material::diffuse(hex_color(0xAAAAAA))),
        );

        let camera = Camera::look_at(
            glm::vec3(0.0, 1.0, 6.0),
            glm::vec3(0.0, 1.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
            std::f64::consts::FRAC_PI_4,
        )
        .focus(glm::vec3(0.0, 1.0, 0.0), 0.02);

        if TEST {
            Renderer::new(&scene, camera)
                .width(200)
                .height(150)
                .max_bounces(7)
                .num_samples(1)
                .render()
                .save(format!("video/image_{}.png", frame))?;
        } else {
            Renderer::new(&scene, camera)
                .width(800)
                .height(600)
                .max_bounces(9)
                .num_samples(2000)
                .render()
                .save(format!("video/image_{}.png", frame))?;
        }
        system.rk4_integrate(&mut cur_state, 1. / 16., 1. / 10000.);
        println!("Frame {} finished", frame);
    }
    Command::new("ffmpeg")
        .args(&["-y", "-i", "video/image_%d.png", "-vcodec", "libx264"])
        .args(&["-s", "800x600", "-pix_fmt", "yuv420p", "video.mp4"])
        .spawn()?
        .wait()?;

    Ok(())
}
