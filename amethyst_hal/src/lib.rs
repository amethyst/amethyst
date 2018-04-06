//!

// #![deny(missing_docs)]
#![deny(unused_must_use)]

extern crate amethyst_assets;
extern crate amethyst_core as core;
extern crate crossbeam_channel;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate gfx_hal as hal;
extern crate gfx_memory as mem;
extern crate imagefmt;
#[macro_use]
extern crate log;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

extern crate shred;
extern crate smallvec;
extern crate specs;
extern crate winit;
extern crate xfg;

#[cfg(feature = "gfx-dx12")]
extern crate gfx_backend_dx12 as dx12;
#[cfg(not(any(feature = "gfx-vulkan", feature = "gfx-metal", feature = "gfx-dx12")))]
extern crate gfx_backend_empty as empty;
#[cfg(feature = "gfx-metal")]
extern crate gfx_backend_metal as metal;
#[cfg(feature = "gfx-vulkan")]
extern crate gfx_backend_vulkan as vulkan;

extern crate wavefront_obj;

error_chain!{}

mod escape;
mod reclamation;
mod uploader;
mod utils;

pub mod assets;
pub mod backend;
pub mod bundle;
pub mod factory;
pub mod light;
pub mod mesh;
pub mod renderer;
pub mod system;
pub mod texture;
pub mod vertex;

pub use bundle::RenderBundle;
pub use factory::{Buffer, Factory, Image};
pub use system::RenderSystem;
