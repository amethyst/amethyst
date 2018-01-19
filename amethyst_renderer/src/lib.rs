//!
//! Rendering engine for Amethyst.
//!

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![deny(unused_must_use)]

extern crate amethyst_assets as assets;
extern crate amethyst_core as core;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate failure;
pub extern crate gfx_hal;
extern crate hibitset;
extern crate imagefmt;
extern crate mint;
extern crate rayon;
extern crate rayon_core;
#[macro_use]
extern crate serde;
extern crate shred;
extern crate smallvec;
extern crate specs;
extern crate winit;

#[cfg(feature = "gfx-backend-vulkan")]
pub extern crate gfx_backend_vulkan as vulkan;

#[cfg(feature = "gfx-backend-metal")]
pub extern crate gfx_backend_metal as metal;

#[macro_use]
extern crate thread_profiler;

extern crate wavefront_obj;

pub mod camera;
pub mod cirque;
pub mod command;
pub mod epoch;
pub mod descriptors;
pub mod factory;
pub mod formats;
pub mod hal;
pub mod light;
pub mod material;
pub mod memory;
pub mod mesh;
pub mod relevant;
pub mod graph;
pub mod passes;
pub mod resources;
pub mod stage;
pub mod system;
pub mod texture;
pub mod upload;
pub mod uniform;
pub mod vertex;

mod utils;
