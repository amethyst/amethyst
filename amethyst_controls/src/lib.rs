//! Amethyst control crate.
extern crate amethyst_core;
extern crate amethyst_input;
extern crate amethyst_renderer;
extern crate log;
extern crate winit;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

mod components;
mod bundles;
mod systems;
mod resources;

pub use self::bundles::FlyControlBundle;
pub use self::components::FlyControlTag;
pub use self::systems::{FlyMovementSystem, FreeRotationSystem, CursorHideSystem};
