#![allow(unused_imports)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
#![allow(dead_code)]


extern crate amethyst_core as core;
extern crate cgmath;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate gfx_hal;
extern crate mint;
extern crate rayon;
extern crate rayon_core;
#[macro_use]
extern crate serde;
extern crate smallvec;
extern crate specs;
extern crate winit;

#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as vulkan;

#[cfg(feature = "metal")]
extern crate gfx_backend_metal as metal;


pub mod memory;
pub mod epoch;

pub mod relevant;
pub mod hal;

// pub mod mesh;
// pub mod cam;
// pub mod graph;
// pub mod shaders;
// pub mod texture;
pub mod vertex;
// pub mod uniform;

// pub mod staging;
pub mod utils;

// mod components;
