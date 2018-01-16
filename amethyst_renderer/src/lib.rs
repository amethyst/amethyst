//!
//! Rendering engine for Amethyst.
//!

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
#![allow(dead_code)]

#![deny(unused_must_use)]

extern crate amethyst_core as core;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate hibitset;
extern crate mint;
extern crate rayon;
extern crate rayon_core;
#[macro_use]
extern crate serde;
extern crate shred;
extern crate smallvec;
extern crate specs;
extern crate winit;
pub extern crate gfx_hal;

#[cfg(feature = "gfx-backend-vulkan")]
pub extern crate gfx_backend_vulkan as vulkan;

#[cfg(feature = "gfx-backend-metal")]
pub extern crate gfx_backend_metal as metal;

#[macro_use]
extern crate thread_profiler;

pub mod cam;
pub mod cirque;
pub mod command;
pub mod epoch;
pub mod descriptors;
pub mod hal;
pub mod memory;
pub mod mesh;
pub mod relevant;
pub mod graph;
pub mod shaders;
pub mod stage;
// pub mod system;
pub mod texture;
pub mod upload;
pub mod uniform;
pub mod vertex;

mod components;
mod utils;
