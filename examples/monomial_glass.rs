use image::{
    codecs::hdr::{HdrDecoder, HdrMetadata},
    Rgb,
};
use std::io::BufReader;

use rpt::*;

fn rgb_to_color(rgb: Rgb<f32>) -> Color {
    glm::vec3(rgb.0[0] as f64, rgb.0[1] as f64, rgb.0[2] as f64)
}

fn load_hdr(url: &str) -> color_eyre::Result<Hdri> {
    let reader = ureq::get(url).call()?.into_reader();
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

    let hdri = load_hdr("https://hdrihaven.com/files/hdris/ballroom_2k.hdr")?;

    let mut scene = Scene::new();
    scene.environment = Environment::Hdri(hdri);

    scene.add(
        Object::new(monomial_surface(2f64, 4f64).translate(&glm::vec3(0.0, -1.0, 0.0)))
            .material(Material::metallic(hex_color(0xffffff), 0.0001)),
    );
    scene.add(
        Object::new(
            cube()
                .rotate_y(glm::pi::<f64>() / 6.0)
                .scale(&glm::vec3(0.5, 0.3, 0.4))
                .translate(&glm::vec3(0.4, -0.8, 4.0)),
        )
        .material(Material::specular(hex_color(0xff00ff), 0.5)),
    );
    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.5, 0.5, 0.5))
                .translate(&glm::vec3(1.5, -0.5, 1.0)),
        )
        .material(Material::specular(hex_color(0x0000ff), 0.1)),
    );
    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.5, 0.5, 0.5))
                .translate(&glm::vec3(-1.5, -0.5, 1.0)),
        )
        .material(Material::specular(hex_color(0x00ff00), 0.1)),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::specular(hex_color(0xaaaaaa), 0.5)),
    );

    scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
    scene.add(Light::Point(
        glm::vec3(100.0, 100.0, 100.0),
        glm::vec3(0.0, 5.0, 5.0),
    ));

    /*
    scene.add(Light::Point(
        glm::vec3(100.0, 100.0, 100.0),
        glm::vec3(0.0, 0.0, -20.0),
    ));
     */

    Renderer::new(&scene, Camera::default())
        .width(800)
        .height(600)
        .max_bounces(1)
        .num_samples(100)
        .render()
        .save("output.png")?;

    Ok(())
}
