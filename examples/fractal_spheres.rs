use rpt::*;

fn gen(scene: &mut Scene, x: f32, y: f32, z: f32, rad: f32, mut colors: Vec<u32>) {
    scene.add(
        Object::new(sphere())
            .translate(&glm::vec3(x, y, z))
            .scale(&glm::vec3(rad, rad, rad))
            .material(Material::diffuse(hex_color(*colors.last().unwrap()))));
    colors.pop();
    if rad < 0.125 {
        return;
    }
    let disp = rad * 3f32 / 2f32;
    let dx: [f32; 6] = [disp, -disp, 0.0, 0.0, 0.0, 0.0];
    let dy: [f32; 6] = [0.0, 0.0, disp, -disp, 0.0, 0.0];
    let dz: [f32; 6] = [0.0, 0.0, 0.0, 0.0, disp, -disp];
    for i in 0..6 {
        gen(scene, x + dx[i], y + dy[i], z + dz[i], rad / 2f32, colors.clone());
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    gen(&mut scene, 0.0, 0.0, 0.0, 1.0, vec![0x264653, 0x2A9D8F, 0xE9C46A, 0xF4A261, 0xE76F51]);

    // black background
    scene.background = hex_color(0x000000);

    scene.add(Light::Ambient(glm::vec3(0.2, 0.2, 0.2)));

    scene.add(Light::Point(
        glm::vec3(36.0, 36.0, 36.0),
        glm::vec3(0.0, 5.0, 5.0),
    ));

    Renderer::new(&scene, Camera{center: glm::vec3(0.0, 7.0, 7.0), direction: glm::vec3(0.0, -1.0, -1.0), up: glm::vec3(0.0, 1.0, -1.0), fov: std::f32::consts::FRAC_PI_6})
        .width(800)
        .height(600)
        .render()
        .save("output.png")?;

    Ok(())
}

