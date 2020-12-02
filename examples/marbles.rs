use image::{
    codecs::hdr::{HdrDecoder, HdrMetadata},
    ImageResult, Rgb,
};
use std::io::BufReader;

use rpt::*;
use std::fs::File;
use std::process::Command;
use std::sync::Arc;

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

const TEST: bool = true;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    std::fs::create_dir_all("video")?;

    const N: usize = 16;
    let pos = (0..N)
        .map(|i| {
            glm::vec3(
                (i / 4) as f64 / 4.,
                4. + rand::random::<f64>() * 2.,
                (i % 4) as f64 / 4.,
            )
        })
        .collect();
    let mut cur_state = ParticleState {
        pos: pos,
        vel: vec![glm::vec3(0., 0., 0.); N],
    };
    const R: f64 = 0.1;

    let system = MarblesSystem { radius: R };

    let hdri = load_hdr("https://hdrihaven.com/files/hdris/ballroom_2k.hdr")?;
    let surface_shape =
        Arc::new(load_obj(File::open("examples/monomial.obj")?)?.scale(&glm::vec3(1., 1., 1.)));
    for frame in 0..720 {
        let mut scene = Scene::new();
        if !TEST {
            scene.environment = Environment::Hdri(hdri.clone());
        }

        let glass = Material::clear(1.5, 0.00001);
        //        scene.add(Object::new(surface_shape.clone()).material(glass));
        scene.add(Object::new(monomial_surface(2., 4.)).material(glass));
        let colors = [0x264653, 0x2A9D8F, 0xE9C46A, 0xF4A261, 0xE76F51];
        let surf = monomial_surface(2., 4.);
        for i in 0..N {
            let mut pos = cur_state.pos[i];
            let closest = surf.closest_point(&pos);
            let vec = pos - closest;
            if glm::length(&vec) < R {
                pos = closest + glm::normalize(&vec) * R;
            }
            scene.add(
                Object::new(sphere().scale(&glm::vec3(R, R, R)).translate(&pos))
                    .material(Material::specular(hex_color(colors[i % colors.len()]), 0.1)),
            );
        }
        scene.add(
            Object::new(plane(glm::vec3(0.0, 1.0, 0.0), 0.0))
                .material(Material::specular(hex_color(0xaaaaaa), 0.5)),
        );

        scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
        if !TEST {
            scene.add(Light::Point(
                glm::vec3(100.0, 100.0, 100.0),
                glm::vec3(0.0, 5.0, 5.0),
            ));
        }

        let camera = Camera::look_at(
            glm::vec3(0.0, 1.0, 10.0),
            glm::vec3(0.0, 1.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
            std::f64::consts::FRAC_PI_6,
        );

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
                .max_bounces(3)
                .num_samples(100)
                .render()
                .save(format!("video/image_{}.png", frame))?;
        }
        system.rk4_integrate(&mut cur_state, 1. / 24., 1. / 10000.);
        println!("Frame {} finished", frame);
    }
    Command::new("ffmpeg")
        .args(&["-y", "-i", "video/image_%d.png", "-vcodec", "libx264"])
        .args(&["-s", "800x600", "-pix_fmt", "yuv420p", "video.mp4"])
        .spawn()?
        .wait()?;

    Ok(())
}
