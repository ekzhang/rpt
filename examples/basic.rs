use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    scene.add(Object::new(sphere()));
    scene.add(
        Object::new(
            cube()
                .rotate_y(glm::pi::<f64>() / 6.0)
                .scale(&glm::vec3(0.5, 0.3, 0.4))
                .translate(&glm::vec3(0.4, -0.8, 4.0)),
        )
        .material(Material::diffuse(hex_color(0xff00ff))),
    );
    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.5, 0.5, 0.5))
                .translate(&glm::vec3(1.5, -0.5, 1.0)),
        )
        .material(Material {
            diffuse: hex_color(0x0000ff),
            specular: glm::vec3(0.25, 0.25, 0.25),
            shininess: 10.0,
        }),
    );
    scene.add(
        Object::new(
            sphere()
                .scale(&glm::vec3(0.5, 0.5, 0.5))
                .translate(&glm::vec3(-1.5, -0.5, 1.0)),
        )
        .material(Material {
            diffuse: hex_color(0x00ff00),
            specular: glm::vec3(0.25, 0.25, 0.25),
            shininess: 10.0,
        }),
    );
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::diffuse(hex_color(0xaaaaaa))),
    );

    scene.add(Light::Ambient(glm::vec3(0.1, 0.1, 0.1)));
    scene.add(Light::Point(
        glm::vec3(36.0, 36.0, 36.0),
        glm::vec3(0.0, 5.0, 5.0),
    ));

    // black background
    scene.background = hex_color(0x000000);

    Renderer::new(&scene, Camera::default())
        .width(800)
        .height(600)
        .render()
        .save("output.png")?;

    Ok(())
}
