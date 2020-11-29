//! Ferris the crab - https://rustacean.net/
//!
//! Original model credit (.PLY) goes to Raptori at https://www.thingiverse.com/thing:3414267

use std::fs::File;

use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    scene.add(
        Object::new(
            load_obj(File::open("examples/rustacean.obj")?)?
                .translate(&glm::vec3(0.0, 0.134649, 0.0))
                .scale(&glm::vec3(2.0, 2.4, 2.0)),
        )
        .material(Material::specular(hex_color(0xF84C00), 0.2)),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), 0.0))
            .material(Material::diffuse(hex_color(0xAAAA77))),
    );

    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(2.0, 2.0, 2.0))
                .translate(&glm::vec3(0.0, 20.0, 3.0)),
        )
        .material(Material::light(glm::vec3(1.0, 1.0, 1.0), 160.0)),
    ));

    let camera = Camera::look_at(
        glm::vec3(-1.5, 5.5, 9.0),
        glm::vec3(0.0, 0.9, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_6,
    );
    Renderer::new(&scene, camera)
        .width(800)
        .height(800)
        .max_bounces(3)
        .num_samples(10)
        .render()
        .save("output.png")?;

    Ok(())
}
