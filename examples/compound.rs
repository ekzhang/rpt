//! Compound of five cubes: https://en.wikipedia.org/wiki/Compound_of_five_cubes

use rpt::*;

#[allow(clippy::many_single_char_names)]
fn lamp(x: f64, y: f64, z: f64, r: f64, e: f64) -> Light {
    Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(r, r, r))
                .translate(&glm::vec3(x, y, z)),
        )
        .material(Material::light(glm::vec3(1.0, 1.0, 1.0), e)),
    )
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    // Related to the golden ratio, computed by hand with some geometry
    let magic_angle = ((3.0 * 5.0_f64.sqrt() - 1.0) / 8.0).acos();

    // Five cubes make up the compound; exploit symmetries
    let c_central = Cube;
    let c_green = c_central.rotate(-magic_angle, &glm::vec3(1.0, 1.0, 1.0));
    let c_red = c_green.scale(&glm::vec3(-1.0, 1.0, 1.0));
    let c_blue = c_green.scale(&glm::vec3(1.0, -1.0, 1.0));
    let c_orange = c_red.scale(&glm::vec3(1.0, -1.0, 1.0));

    scene.add(Object::new(c_central).material(Material::specular(hex_color(0xC144EB), 0.4)));
    scene.add(Object::new(c_green).material(Material::specular(hex_color(0x45E542), 0.4)));
    scene.add(Object::new(c_red).material(Material::specular(hex_color(0xF55142), 0.4)));
    scene.add(Object::new(c_blue).material(Material::specular(hex_color(0x4275F5), 0.4)));
    scene.add(Object::new(c_orange).material(Material::specular(hex_color(0xF5BF42), 0.4)));

    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -0.80902))
            .material(Material::diffuse(hex_color(0xFFFFFF))),
    );

    scene.add(lamp(-2.0, 3.5, 0.5, 0.5, 60.0));
    scene.add(lamp(0.0, 0.5, 5.0, 1.0, 2.0));
    scene.add(lamp(2.0, 1.0, -5.0, 0.6, 10.0));

    let camera = Camera::look_at(
        glm::vec3(-0.9, 1.2, 2.4),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_4,
    );
    Renderer::new(&scene, camera)
        .width(1024)
        .height(1024)
        .max_bounces(5)
        .num_samples(50)
        .render()
        .save("output.png")?;

    Ok(())
}
