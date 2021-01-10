//! A collection of useful amethyst utilities, designed to make your game dev life easier.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub use self::app_root_dir::*;

pub mod app_root_dir;
pub mod auto_fov;
pub mod circular_buffer;
pub mod fps_counter;
pub mod ortho_camera;
pub mod removal;
pub mod tag;
pub mod time_destroy;
