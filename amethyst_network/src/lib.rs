//! Provides a client-server networking architecture to amethyst.

#![deny(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate bincode;
extern crate cgmath;
#[macro_use]
extern crate serde;
extern crate shrev;
extern crate specs;

pub mod systems;
pub mod resources;
pub mod components;
pub mod filters;

pub use components::*;
pub use filters::*;
pub use resources::*;
pub use systems::*;
