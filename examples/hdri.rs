//! Demo of using a custom HDRI to light a scene

use image::{
    codecs::hdr::{HdrDecoder, HdrMetadata},
    ImageResult, Rgb,
};
use std::io::BufReader;

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

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let hdri = load_hdr("https://hdrihaven.com/files/hdris/ballroom_2k.hdr")?;

    let mut scene = Scene::new();
    scene.environment = Environment::Hdri(hdri);
    scene.add(
        Object::new(sphere().translate(&glm::vec3(1.1, 0.0, 0.0)))
            .material(Material::metallic(hex_color(0xffffff), 0.0001)),
    );
    scene.add(
        Object::new(sphere().translate(&glm::vec3(-1.1, 0.0, 0.0)))
            .material(Material::metallic(hex_color(0xffffff), 0.1)),
    );

    Renderer::new(&scene, Camera::default())
        .width(1200)
        .height(900)
        .max_bounces(5)
        .num_samples(50)
        .render()
        .save("output.png")?;

    Ok(())
}
