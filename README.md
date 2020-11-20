# rpt: a path tracer in rust

Work in progress — a path tracing engine written in Rust.

The library containing path tracing code is located inside `src/`. Example code and scenes are located in `examples/`. To compile the code and run `examples/basic.rs`, use the command:

```bash
cargo run --example basic
```

To run tests, use:

```bash
cargo test
```

To generate documentation, use:

```bash
cargo doc --open
```

## References

- https://github.com/fogleman/pt
- http://www.pbr-book.org/
- https://www.scratchapixel.com/lessons/3d-basic-rendering/global-illumination-path-tracing
- https://en.wikipedia.org/wiki/Gamma_correction
