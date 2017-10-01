//! A data parallel rendering engine developed by the [Amethyst][am] project.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [am]: https://www.amethyst.rs/
//! [gh]: https://github.com/amethyst/amethyst/tree/develop/src/renderer
//! [bk]: https://www.amethyst.rs/book/

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate cgmath;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate gfx;
extern crate gfx_core;
#[macro_use]
extern crate gfx_macros;
extern crate hetseq;
extern crate num_cpus;
extern crate rayon;
extern crate rayon_core;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate specs;
extern crate winit;

#[cfg(all(feature = "d3d11", target_os = "windows"))]
extern crate gfx_device_dx11;
#[cfg(all(feature = "d3d11", target_os = "windows"))]
extern crate gfx_window_dxgi;

#[cfg(all(feature = "metal", target_os = "macos"))]
extern crate gfx_device_metal;
#[cfg(all(feature = "metal", target_os = "macos"))]
extern crate gfx_window_metal;

#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

#[cfg(feature = "vulkan")]
extern crate gfx_device_vulkan;
#[cfg(feature = "vulkan")]
extern crate gfx_window_vulkan;

pub use cam::{Camera, Projection};
pub use color::Rgba;
pub use config::Config;
pub use error::{Error, Result};
pub use light::Light;
pub use mesh::{Mesh, MeshBuilder};
pub use mtl::{Material, MaterialBuilder};
pub use pipe::{PipelineBuilder, PolyPipeline, PolyStage, Target};
pub use renderer::Renderer;
pub use tex::{Texture, TextureBuilder};
pub use types::{Encoder, Factory};
pub use vertex::VertexFormat;

pub mod light;
pub mod pass;
pub mod prelude;
pub mod pipe;
pub mod vertex;
pub mod types;

mod cam;
mod color;
mod config;
mod error;
mod mesh;
mod mtl;
mod renderer;
mod tex;
