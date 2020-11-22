use image::{
    codecs::hdr::{HdrDecoder, HdrMetadata},
    ImageResult, Rgb,
};
use std::io::BufReader;
use std::time::Instant;

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
        Object::new(load_obj("examples/wine_glass.obj")?).material(Material::clear(1.5, 0.0001)),
    );
    scene.add(
        Object::new(polygon(&[
            glm::vec3(-10.0, 0.0, -10.0),
            glm::vec3(-10.0, 0.0, 10.0),
            glm::vec3(10.0, 0.0, 10.0),
            glm::vec3(10.0, 0.0, -10.0),
        ]))
        .material(Material::diffuse(hex_color(0x6f5d48))),
    );

    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(3.0, 3.0, 3.0))
                .translate(&glm::vec3(11.15, 13.739, -4.9325)),
        )
        .material(Material::light(hex_color(0xFFFFFF), 200.0)),
    );

    let eye = glm::vec3(5.530, 4.375, 5.384);
    let camera = Camera::look_at(
        eye,
        eye + glm::vec3(-0.6962, -0.3754, -0.6119),
        glm::vec3(0.0, 1.0, 0.0),
        0.6911,
    );

    let mut time = Instant::now();
    Renderer::new(&scene, camera)
        .width(960)
        .height(540)
        .max_bounces(6)
        .num_samples(100)
        .filter(Filter::Box(1))
        .iterative_render(10, |iteration, buffer| {
            let millis = time.elapsed().as_millis();
            println!(
                "Finished iteration {}, took {} ms, variance: {}",
                iteration,
                millis,
                buffer.variance()
            );
            buffer
                .image()
                .save(format!("output_{:03}.png", iteration - 1))
                .expect("Failed to save image");
            time = Instant::now();
        });

    Ok(())
}
