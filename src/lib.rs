//! `rpt` is a path tracer in Rust.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use color::*;
pub use light::*;
pub use material::*;
pub use object::*;
pub use renderer::*;
pub use scene::*;
pub use shape::*;
pub use transform::*;

mod color;
mod light;
mod material;
mod object;
mod renderer;
mod scene;
mod shape;
mod transform;
