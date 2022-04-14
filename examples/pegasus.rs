//! Ice sculpture of a horse
//!
//! Source: https://www.cgtrader.com/free-3d-print-models/art/sculptures/pegasus-statue-sculpture-statuette-figurine-horse

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Cursor, SeekFrom};
use std::time::Instant;

use image::codecs::hdr::{HdrDecoder, HdrMetadata};
use image::Rgb;
use rpt::*;
use tempfile::tempfile;
use zip::ZipArchive;

fn load_pegasus() -> color_eyre::Result<Mesh> {
    let mut buf = Vec::new();
    File::open("examples/pegasus.zip")?.read_to_end(&mut buf)?;
    let mut archive = ZipArchive::new(Cursor::new(buf))?;
    let mut make_tempfile = |name| {
        let mut buf = Vec::new();
        archive.by_name(name)?.read_to_end(&mut buf)?;
        let mut file = tempfile()?;
        file.write_all(&buf)?;
        file.seek(SeekFrom::Start(0))?;
        Ok::<_, color_eyre::Report>(file)
    };
    let obj_file = make_tempfile("pegasus.obj")?;
    load_obj(obj_file).map_err(|e| e.into())
}

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

    let hdri = load_hdr("https://hdrihaven.com/files/hdris/birchwood_8k.hdr")?;

    let pegasus = load_pegasus()?;
    let ice = Material::transparent(hex_color(0xF8F8FF), 1.31, 0.2);

    let mut scene = Scene::new();
    scene.add(Object::new(pegasus.scale(&glm::vec3(1.4, 1.4, 1.4))).material(ice));
    scene.add(
        Object::new(polygon(&[
            glm::vec3(2.0, -0.01, 2.0),
            glm::vec3(2.0, -0.01, -2.0),
            glm::vec3(-2.0, -0.01, -2.0),
            glm::vec3(-2.0, -0.01, 2.0),
        ]))
        .material(Material::diffuse(hex_color(0xDDDDDD))),
    );

    scene.environment = Environment::Hdri(hdri);

    let camera = Camera::look_at(
        glm::vec3(0.0, 1.5, 3.1),
        glm::vec3(0.0, 1.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_4,
    );

    let mut time = Instant::now();
    Renderer::new(&scene, camera)
        .width(1200)
        .height(1200)
        .exposure_value(-1.5)
        .max_bounces(8)
        .num_samples(10)
        .iterative_render(1, |iteration, buffer| {
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
