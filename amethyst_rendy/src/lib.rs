// this is temporary
// #![allow(dead_code)]
// #![allow(unused_variables)]

#[macro_use]
extern crate amethyst_derive;

#[macro_use]
extern crate shred_derive;

pub use palette;
pub use rendy;

pub mod pass;

pub mod batch;
pub mod camera;
pub mod error;
pub mod formats;
pub mod hidden;
pub mod light;
pub mod mtl;
pub mod resources;
pub mod shape;
pub mod skinning;
pub mod sprite;
pub mod sprite_visibility;
pub mod system;
pub mod transparent;
pub mod types;
pub mod visibility;

mod pod;
