use rpt::*;
use std::process::Command;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;



    std::fs::create_dir("video");

    for i in 0..60 {
        let mut scene = Scene::new();

        scene.add(Object::new(sphere()));
        scene.add(
            Object::new(
                cube()
                    .rotate_y(glm::pi::<f64>() / 6.0)
                    .scale(&glm::vec3(0.5, 0.3, 0.4))
                    .translate(&glm::vec3(0.4, -0.8, 4.0 + 0.01 * i as f64)),
            )
                .material(Material::specular(hex_color(0xff00ff), 0.5)),
        );
        scene.add(
            Object::new(
                sphere()
                    .scale(&glm::vec3(0.5, 0.5, 0.5))
                    .translate(&glm::vec3(1.5, -0.5, 1.0)),
            )
                .material(Material::specular(hex_color(0x0000ff), 0.1)),
        );
        scene.add(
            Object::new(
                sphere()
                    .scale(&glm::vec3(0.5, 0.5, 0.5))
                    .translate(&glm::vec3(-1.5, -0.5, 1.0)),
            )
                .material(Material::specular(hex_color(0x00ff00), 0.1)),
        );
        scene.add(
            Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
                .material(Material::specular(hex_color(0xaaaaaa), 0.5)),
        );

        scene.add(Light::Ambient(glm::vec3(0.01, 0.01, 0.01)));
        scene.add(Light::Point(
            glm::vec3(100.0, 100.0, 100.0),
            glm::vec3(0.0, 5.0, 5.0),
        ));
        Renderer::new(&scene, Camera::default())
            .width(800)
            .height(600)
            .num_samples(100)
            .max_bounces(1)
            .render()
            .save(format!("video/image_{}.png", i))?;
    }
    Command::new("ffmpeg").args(&["-y", "-i", "video/image_%d.png", "-vcodec", "libx264", "-s", "800x600", "-pix_fmt", "yuv420p", "video.mp4"]).spawn().unwrap().wait();

    Ok(())
}
