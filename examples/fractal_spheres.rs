use rpt::*;

fn gen(
    scene: &mut Scene,
    x: f32,
    y: f32,
    z: f32,
    rad: f32,
    colors: &[u32],
    depth: usize,
    last_dir: Option<usize>,
) {
    scene.add(
        Object::new(sphere())
            .translate(&glm::vec3(x, y, z))
            .scale(&glm::vec3(rad, rad, rad))
            .material(Material::diffuse(hex_color(colors[depth]))),
    );
    if depth == colors.len() - 1 {
        return;
    }
    let disp = rad * 7.0 / 5.0;
    let dx: [f32; 6] = [disp, -disp, 0.0, 0.0, 0.0, 0.0];
    let dy: [f32; 6] = [0.0, 0.0, disp, -disp, 0.0, 0.0];
    let dz: [f32; 6] = [0.0, 0.0, 0.0, 0.0, disp, -disp];
    for i in 0..6 {
        if last_dir.is_none() || i != (last_dir.unwrap() ^ 1) {
            gen(
                scene,
                x + dx[i],
                y + dy[i],
                z + dz[i],
                rad * 2.0 / 5.0,
                &colors,
                depth + 1,
                Some(i),
            );
        }
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    let colors = [0x264653, 0x2A9D8F, 0xE9C46A, 0xF4A261, 0xE76F51];
    gen(&mut scene, 0.0, 0.0, 0.0, 1.0, &colors, 0, None);

    // black background
    scene.background = hex_color(0x000000);

    scene.add(Light::Ambient(glm::vec3(0.2, 0.2, 0.2)));
    scene.add(Light::Directional(
        glm::vec3(0.2, 0.2, 0.2),
        glm::vec3(0.0, -0.65, -1.0).normalize(),
    ));
    scene.add(Light::Point(
        glm::vec3(36.0, 36.0, 36.0),
        glm::vec3(0.0, 5.0, 5.0),
    ));

    let camera = Camera {
        center: glm::vec3(2.0, 3.5, 7.0),
        direction: glm::vec3(-0.285714, -0.5, -1.0).normalize(),
        up: glm::vec3(0.0, 1.0, -0.5).normalize(),
        fov: std::f32::consts::FRAC_PI_6,
    };
    Renderer::new(&scene, camera)
        .width(800)
        .height(600)
        .render()
        .save("output.png")?;

    Ok(())
}
