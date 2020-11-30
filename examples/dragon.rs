//! Stanford dragon render

use std::io::{prelude::*, Cursor, SeekFrom};
use tempfile::tempfile;
use zip::ZipArchive;

use rpt::*;

fn load_dragon() -> color_eyre::Result<Mesh> {
    let mut buf = Vec::new();
    ureq::get("http://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip")
        .call()
        .into_reader()
        .read_to_end(&mut buf)?;
    let mut archive = ZipArchive::new(Cursor::new(buf))?;
    let mut buf = Vec::new();
    archive.by_name("dragon.obj")?.read_to_end(&mut buf)?;
    let mut file = tempfile()?;
    file.write_all(&buf)?;
    file.seek(SeekFrom::Start(0))?;
    load_obj(file).map_err(|e| e.into())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    println!("Loading dragon...");
    let dragon = load_dragon()?;
    println!("Finished loading dragon!");

    let mut scene = Scene::new();
    scene.add(
        Object::new(
            dragon
                .scale(&glm::vec3(3.4, 3.4, 3.4))
                .rotate_y(std::f64::consts::FRAC_PI_2),
        )
        .material(Material::specular(hex_color(0xB7CA79), 0.1)),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::diffuse(hex_color(0xAAAAAA))),
    );
    scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(2.0, 2.0, 2.0))
                .translate(&glm::vec3(0.0, 20.0, 3.0)),
        )
        .material(Material::light(glm::vec3(1.0, 1.0, 1.0), 160.0)),
    ));
    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.05, 0.05, 0.05))
                .translate(&glm::vec3(-1.0, 0.71, 0.0)),
        )
        .material(Material::light(hex_color(0xFFAAAA), 400.0)),
    ));

    let camera = Camera::look_at(
        glm::vec3(-2.5, 4.0, 6.5),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_6,
    );
    Renderer::new(&scene, camera)
        .max_bounces(2)
        .num_samples(1)
        .render()
        .save("output.png")?;

    Ok(())
}
