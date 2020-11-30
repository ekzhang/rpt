use image::{
    codecs::hdr::{HdrDecoder, HdrMetadata},
    ImageResult, Rgb,
};
use std::io::BufReader;

use rpt::*;
use std::process::Command;

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

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    std::fs::create_dir_all("video")?;

    let mut cur_state = ParticleState {
        pos: vec![glm::vec3(0.1, 3., 0.2), glm::vec3(0.5, 4.5, 0.1)],
        vel: vec![glm::vec3(0., 0., 0.); 2],
    };
    const R: f64 = 0.1;

    let system = MarblesSystem;

    let hdri = load_hdr("https://hdrihaven.com/files/hdris/ballroom_2k.hdr")?;
    for frame in 0..120 {
        let mut scene = Scene::new();
        scene.environment = Environment::Hdri(hdri.clone());

        let glass = Material::clear(1.5, 0.00001);
        scene.add(Object::new(monomial_surface(2f64, 4f64)).material(glass));
        scene.add(
            Object::new(
                sphere()
                    .scale(&glm::vec3(R, R, R))
                    .translate(&cur_state.pos[0]),
            )
            .material(Material::specular(hex_color(0x0000ff), 0.1)),
        );
        scene.add(
            Object::new(
                sphere()
                    .scale(&glm::vec3(R, R, R))
                    .translate(&cur_state.pos[1]),
            )
            .material(Material::specular(hex_color(0x00ff00), 0.1)),
        );
        scene.add(
            Object::new(plane(glm::vec3(0.0, 1.0, 0.0), 0.0))
                .material(Material::specular(hex_color(0xaaaaaa), 0.5)),
        );

        scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
        /*
        scene.add(Light::Point(
            glm::vec3(100.0, 100.0, 100.0),
            glm::vec3(0.0, 5.0, 5.0),
        ));
        */

        Renderer::new(
            &scene,
            Camera {
                eye: glm::vec3(0.0, 1.0, 10.0),
                direction: glm::vec3(0.0, 0.0, -1.0),
                up: glm::vec3(0.0, 1.0, 0.0),
                fov: std::f64::consts::FRAC_PI_6,
            },
        )
        .width(200)
        .height(150)
        .max_bounces(3)
        .num_samples(1)
        .render()
        .save(format!("video/image_{}.png", frame))?;
        system.rk4_integrate(&mut cur_state, 1. / 24., 1. / 1000.);
        println!("Frame {} finished", frame);
    }
    Command::new("ffmpeg")
        .args(&["-y", "-i", "video/image_%d.png", "-vcodec", "libx264"])
        .args(&["-s", "800x600", "-pix_fmt", "yuv420p", "video.mp4"])
        .spawn()?
        .wait()?;

    Ok(())
}
