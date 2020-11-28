//! Lego creator plane model, source https://free3d.com/3d-model/lego-creator-plane-4953-24788.html

use std::fs::File;
use std::io::{prelude::*, Cursor, SeekFrom};
use std::time::Instant;
use tempfile::tempfile;
use zip::ZipArchive;

use rpt::*;

fn load_lego_plane() -> color_eyre::Result<Vec<Object>> {
    let mut buf = Vec::new();
    File::open("examples/lego.zip")?.read_to_end(&mut buf)?;
    let mut archive = ZipArchive::new(Cursor::new(buf))?;
    println!(
        "Zip has contents: {:?}",
        archive.file_names().collect::<Vec<_>>()
    );
    let mut make_tempfile = |name| {
        let mut buf = Vec::new();
        archive.by_name(name)?.read_to_end(&mut buf)?;
        let mut file = tempfile()?;
        file.write_all(&buf)?;
        file.seek(SeekFrom::Start(0))?;
        Ok::<_, color_eyre::Report>(file)
    };
    let obj_file = make_tempfile("LEGO.Creator_Plane/LEGO.Creator_Plane.obj")?;
    let mtl_file = make_tempfile("LEGO.Creator_Plane/LEGO.Creator_Plane.mtl")?;
    load_obj_with_mtl(obj_file, mtl_file).map_err(|e| e.into())
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    println!("Loading lego plane...");
    let lego_plane = load_lego_plane()?;
    println!("Finished loading lego plane!");

    let mut scene = Scene::new();
    for mut object in lego_plane {
        // This is a bit of a hack, since we don't have a way to transform objects
        object.shape = Box::new(
            object
                .shape
                .scale(&glm::vec3(0.002, 0.002, 0.002))
                .translate(&glm::vec3(-0.720, -0.243, -0.770)),
        );
        scene.add(object);
    }

    scene.add(Light::Object(
        Object::new(
            sphere()
                .scale(&glm::vec3(6.0, 6.0, 6.0))
                .translate(&glm::vec3(0.0, 20.0, 30.0)),
        )
        .material(Material::light(glm::vec3(1.0, 1.0, 1.0), 25.0)),
    ));

    let camera = Camera::look_at(
        glm::vec3(2.5, 2.0, 1.5),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        std::f64::consts::FRAC_PI_6,
    );

    let mut time = Instant::now();
    Renderer::new(&scene, camera)
        .width(960)
        .height(540)
        .max_bounces(5)
        .num_samples(20)
        .iterative_render(1, |iteration, buffer| {
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
