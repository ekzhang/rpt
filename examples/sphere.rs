use rpt::*;

fn main() {
    let mut scene = Scene::new();

    scene.add(Object::new(sphere())); // default red material
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::diffuse(hex_color(0xaaaaaa))),
    );
    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(2.0, 2.0, 2.0))
                .translate(&glm::vec3(0.0, 12.0, 0.0)),
        )
        .material(Material::light(hex_color(0xFFFFFF), 40.0)),
    ));

    let camera = Camera::look_at(
        glm::vec3(-2.5, 4.0, 6.5),
        glm::vec3(0.0, -0.25, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_4,
    );

    Renderer::new(&scene, camera)
        .width(960)
        .height(540)
        .max_bounces(2)
        .num_samples(100)
        .render()
        .save("output.png")
        .unwrap();
}
