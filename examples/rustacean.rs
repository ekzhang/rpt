//! Ferris the crab - https://rustacean.net/
//!
//! Original model credit (.PLY) goes to Raptori at https://www.thingiverse.com/thing:3414267

use std::fs::File;

use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    let crab_scale = glm::vec3(2.0, 2.4, 2.0);
    scene.add(
        Object::new(
            load_obj(File::open("examples/rustacean.obj")?)?
                .translate(&glm::vec3(0.0, 0.134649, 0.0))
                .scale(&crab_scale),
        )
        .material(Material::specular(hex_color(0xF84C00), 0.2)),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), 0.0))
            .material(Material::diffuse(hex_color(0xAAAA77))),
    );

    let balls = &[
        (true, 0.2, glm::vec3(-0.81, 1.02, 0.47)),
        (true, 0.3, glm::vec3(-0.86, 1.10, 0.36)),
        (true, 0.4, glm::vec3(-0.75, 1.12, 0.34)),
        (false, 0.2, glm::vec3(0.87, 1.03, 0.41)),
        (false, 0.3, glm::vec3(0.75, 1.09, 0.36)),
        (false, 0.4, glm::vec3(0.85, 1.15, 0.45)),
    ];
    for &(glass, roughness, pos) in balls {
        let pos = crab_scale.component_mul(&pos);
        scene.add(
            Object::new(sphere().scale(&glm::vec3(0.1, 0.1, 0.1)).translate(&pos)).material(
                if glass {
                    Material::clear(1.5, roughness)
                } else {
                    Material::metallic(hex_color(0xFFFFFF), roughness)
                },
            ),
        );
    }

    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(2.0, 2.0, 2.0))
                .translate(&glm::vec3(0.0, 20.0, 3.0)),
        )
        .material(Material::light(glm::vec3(1.0, 1.0, 1.0), 160.0)),
    ));

    let camera = Camera::look_at(
        glm::vec3(-2.5, 4.0, 8.5),
        glm::vec3(0.0, 0.9, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_6,
    );
    Renderer::new(&scene, camera)
        .width(800)
        .height(800)
        .max_bounces(4)
        .num_samples(10)
        .render()
        .save("output.png")?;

    Ok(())
}
