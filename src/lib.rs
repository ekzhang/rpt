//! `rpt` is a path tracer in Rust.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use color::*;
pub use renderer::*;
pub use scene::*;

mod color;
mod renderer;
mod scene;
