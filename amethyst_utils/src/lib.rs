//! A collection of useful amethyst utilities, designed to make your game dev life easier.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

extern crate amethyst_assets;
extern crate amethyst_controls;
extern crate amethyst_core;
#[macro_use]
extern crate amethyst_derive;
extern crate amethyst_renderer;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate shred;
#[macro_use]
extern crate shred_derive;

pub mod app_root_dir;
pub mod circular_buffer;
pub mod fps_counter;
pub mod ortho_camera;
pub mod removal;
pub mod scene;
pub mod tag;
pub mod time_destroy;
pub use self::app_root_dir::*;
