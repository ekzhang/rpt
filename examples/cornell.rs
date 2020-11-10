//! This is an example of a standard Cornell box, for testing global illumination
//!
//! Reference: https://www.graphics.cornell.edu/online/box/data.html

use std::time::Instant;

use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    let camera = Camera {
        center: glm::vec3(278.0, 273.0, -800.0),
        direction: glm::vec3(0.0, 0.0, 1.0),
        up: glm::vec3(0.0, 1.0, 0.0),
        fov: 0.686,
    };

    let white = Material::diffuse(hex_color(0xAAAAAA));
    let red = Material::diffuse(hex_color(0xBC0000));
    let green = Material::diffuse(hex_color(0x00BC00));
    let light_mtl = Material::light(hex_color(0xFFFEFA), 100.0); // 6500 K

    let floor = plane(glm::vec3(0.0, 1.0, 0.0), 0.0);
    let ceiling = plane(glm::vec3(0.0, 1.0, 0.0), 548.9);
    let light_rect = polygon(&[
        glm::vec3(343.0, 548.8, 227.0),
        glm::vec3(343.0, 548.8, 332.0),
        glm::vec3(213.0, 548.8, 332.0),
        glm::vec3(213.0, 548.8, 227.0),
    ]);
    let back_wall = plane(glm::vec3(0.0, 0.0, 1.0), 559.2);
    let right_wall = plane(glm::vec3(1.0, 0.0, 0.0), 0.0);
    let left_wall = plane(glm::vec3(1.0, 0.0, 0.0), 556.0);

    let large_box = cube()
        .scale(&glm::vec3(165.0, 330.0, 165.0))
        .rotate_y(glm::two_pi::<f64>() * (-253.0 / 360.0))
        .translate(&glm::vec3(368.0, 165.0, 351.0));
    let small_box = cube()
        .scale(&glm::vec3(165.0, 165.0, 165.0))
        .rotate_y(glm::two_pi::<f64>() * (-197.0 / 360.0))
        .translate(&glm::vec3(185.0, 82.5, 169.0));

    scene.add(Object::new(floor).material(white));
    scene.add(Object::new(ceiling).material(white));
    scene.add(Object::new(back_wall).material(white));
    scene.add(Object::new(left_wall).material(red));
    scene.add(Object::new(right_wall).material(green));
    scene.add(Object::new(light_rect).material(light_mtl));
    scene.add(Object::new(large_box).material(white));
    scene.add(Object::new(small_box).material(white));

    let mut time = Instant::now();
    Renderer::new(&scene, camera)
        .width(1024)
        .height(1024)
        .max_bounces(2)
        .num_samples(100)
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
