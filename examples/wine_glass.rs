use std::time::Instant;

use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    scene.add(
        Object::new(
            load_obj("examples/wine_glass.obj")?
                .scale(&glm::vec3(0.2, 0.2, 0.2))
                .translate(&glm::vec3(0.0, -1.12, 0.0)),
        )
        .material(Material::clear(1.5, 0.01)),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::diffuse(hex_color(0xaaaaaa))),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), 1.0))
            .material(Material::diffuse(hex_color(0xaaaaaa))),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 0.0, 1.0), -1.0))
            .material(Material::diffuse(hex_color(0xaaaaaa))),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 0.0, 1.0), 3.0))
            .material(Material::diffuse(hex_color(0xaaaaaa))),
    );
    scene.add(
        Object::new(plane(glm::vec3(1.0, 0.0, 0.0), -1.0))
            .material(Material::diffuse(hex_color(0xff0000))),
    );
    scene.add(
        Object::new(plane(glm::vec3(1.0, 0.0, 0.0), 1.0))
            .material(Material::diffuse(hex_color(0x00ff00))),
    );

    scene.add(Light::Object(
        Object::new(polygon(&[
            glm::vec3(-0.25, 0.99, -0.25),
            glm::vec3(0.25, 0.99, -0.25),
            glm::vec3(0.25, 0.99, 0.25),
            glm::vec3(-0.25, 0.99, 0.25),
        ]))
        .material(Material::light(hex_color(0xFFFEFA), 40.0)),
    ));

    let camera = Camera::look_at(
        glm::vec3(0.0, 0.0, 2.95),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        glm::quarter_pi(),
    );

    let mut time = Instant::now();
    Renderer::new(&scene, camera)
        .width(800)
        .height(800)
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
