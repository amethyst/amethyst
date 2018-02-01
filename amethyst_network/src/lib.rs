//! Provides a client-server networking architecture to amethyst.

#![deny(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate shrev;
#[macro_use]
extern crate log;
extern crate specs;

pub mod components;
pub mod filters;
pub mod resources;
pub mod systems;

pub use components::*;
pub use filters::*;
pub use resources::*;
pub use systems::*;
