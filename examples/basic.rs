use rpt::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    scene.add(Object::new(sphere()));
    scene.add(Light::Ambient(glm::vec3(0.2, 0.2, 0.2)));
    scene.add(Light::Point(
        glm::vec3(36.0, 36.0, 36.0),
        glm::vec3(0.0, 5.0, 5.0),
    ));

    // magenta background
    scene.background = hex_color(0xff00ff);

    Renderer::new(&scene, Camera::default())
        .width(800)
        .height(600)
        .render()
        .save("output.png")?;

    Ok(())
}
