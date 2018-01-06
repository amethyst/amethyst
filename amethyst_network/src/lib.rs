//! Provides a client-server networking architecture to amethyst.

//#![deny(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate cgmath;
extern crate serde;
extern crate shrev;
extern crate specs;

pub mod systems;
pub mod resources;
pub mod components;

pub use systems::*;
pub use resources::*;
pub use components::*;