//! Provides a client-server networking architecture to amethyst.

//#![deny(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate shrev;
#[macro_use]
extern crate log;
extern crate specs;
extern crate uuid;
extern crate rand;

pub mod components;
mod filter;
pub mod resources;
pub mod systems;
mod bundle;

pub use components::*;
pub use filter::*;
pub use resources::*;
pub use systems::*;
pub use bundle::NetworkClientBundle;
