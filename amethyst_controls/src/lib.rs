//! Amethyst control crate.
extern crate amethyst_core;
extern crate amethyst_input;
extern crate amethyst_renderer;
extern crate amethyst_utils;
#[macro_use]
extern crate log;
extern crate shred;
extern crate specs;
extern crate winit;

mod components;
mod bundles;
mod systems;

pub use self::bundles::FlyControlBundle;
pub use self::components::FlyControlTag;
pub use self::systems::{FlyMovementSystem, FreeRotationSystem, MouseCenterLockSystem};
