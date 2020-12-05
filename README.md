# rpt

[![Latest Version](https://img.shields.io/crates/v/rpt.svg)](https://crates.io/crates/rpt)
[![API Documentation](https://docs.rs/rpt/badge.svg)](https://docs.rs/rpt)

This is a physically based, CPU-only rendering engine written in Rust. It uses path tracing to generate realistic images of 3D scenes.

![Demo renders](https://i.imgur.com/3J6B3cx.jpg)
![Demo video](https://i.imgur.com/UTS3Yo8.gif)

## Features

- Simple declarative API, 100% Safe Rust
- Supports .OBJ, .MTL, and .STL file formats
- Uses unbiased path tracing for physically-based light transport
- Uses a microfacet BSDF model with multiple importance sampling
- Uses kd-trees to accelerate ray intersections
- Supports direct light sampling and emissive materials
- Supports HDRI environment maps
- Supports depth of field
- Supports iterative rendering, variance estimation, and firefly reduction
- Supports physics simulation with numerical integrators and particle systems
- Uses all CPU cores concurrently, scaling linearly up to 96 cores

## Quickstart

First, clone the repository. The library containing path tracing code is located inside `src/`. Example code and scenes are located in `examples/`. To compile and run `examples/basic.rs`, use the command:

```bash
cargo run --example basic
```

To run tests, use:

```bash
cargo test
```

### Library Usage

To use `rpt` as a library, add the following to your `Cargo.toml`:

```toml
[dependencies]
rpt = "0.1"
```

Here's a simple scene that demonstrates the basics of the API.

```rust
use rpt::*;

fn main() {
    let mut scene = Scene::new();

    scene.add(Object::new(sphere())); // default red material
    scene.add(
        Object::new(plane(glm::vec3(0.0, 1.0, 0.0), -1.0))
            .material(Material::diffuse(hex_color(0xAAAAAA))),
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
```

![Example output](https://i.imgur.com/RioQyXf.png)

This code can also be found in `examples/sphere.rs`. Note that the shadow is correctly tinted red due to [global illumination](https://en.wikipedia.org/wiki/Global_illumination). See the detailed [API documentation](https://docs.rs/rpt) for information about all of the features, and feel free to learn from the other examples!

## References

- [Physically Based Rendering, 3rd ed.](http://www.pbr-book.org/)
- [CS 348B: Image Synthesis Techniques](https://graphics.stanford.edu/courses/cs348b/)
- [Microfacet Models for Refraction through Rough Surfaces](https://www.graphics.cornell.edu/~bjw/microfacetbsdf.pdf)
- [Scratchapixel: Global Illumination and Path Tracing](https://www.scratchapixel.com/lessons/3d-basic-rendering/global-illumination-path-tracing)
- [SIGGRAPH '10 Notes on Physically-Based Shading](https://renderwonk.com/publications/s2010-shading-course/hoffman/s2010_physically_based_shading_hoffman_a_notes.pdf)
- Inspired by: [fogleman/pt](https://github.com/fogleman/pt), [hunterloftis/pbr](https://github.com/hunterloftis/pbr)

## Samples

![Dragon](https://i.imgur.com/UEWtPDi.png)
![Cornell box](https://i.imgur.com/K7H8rz4.png)
![Pegasus](https://i.imgur.com/sBKAboG.png)
![Lego plane](https://i.imgur.com/BMVCnZ7.png)
![Fractal spheres](https://i.imgur.com/4aO9A2o.png)
![Rustacean](https://i.imgur.com/zZgl7jE.png)
![Wine glass](https://i.imgur.com/8EAmwuq.png)
![Spheres](https://i.imgur.com/jyNpLN5.png)

## Acknowledgements

This project was built by [Eric Zhang](https://github.com/ekzhang) and [Alexander Morozov](https://github.com/scanhex). We'd like to thank [Justin Solomon](https://people.csail.mit.edu/jsolomon/), Yuanming Hu, Lingxiao Li, and Dmitriy Smirnov for teaching an excellent computer graphics class at MIT.

Some of the examples use free 3D models and image assets available on the Internet. Links are provided in comments in the source code, where used.
