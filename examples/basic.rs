use color_eyre::eyre::Result;
use rpt::*;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut scene = Scene::new();

    // magenta background
    scene.background = hex_color(0xff00ff);

    Renderer::new(&scene)
        .width(800)
        .height(600)
        .render()
        .save("output.png")?;

    Ok(())
}
