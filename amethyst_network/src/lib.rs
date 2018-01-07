//! Provides a client-server networking architecture to amethyst.

//#![deny(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate cgmath;
#[macro_use]
extern crate serde;
extern crate shrev;
extern crate specs;
extern crate ron;

pub mod systems;
pub mod resources;
pub mod components;

pub use systems::*;
pub use resources::*;
pub use components::*;