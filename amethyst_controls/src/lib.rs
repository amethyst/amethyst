//! Amethyst control crate.
extern crate amethyst_core;
extern crate amethyst_input;
extern crate amethyst_renderer;
#[macro_use]
extern crate log;
extern crate winit;
extern crate shrev;

#[cfg(feature = "profiler")]
extern crate thread_profiler;

mod components;
mod bundles;
mod systems;
mod resources;

pub use self::bundles::FlyControlBundle;
pub use self::components::FlyControlTag;
pub use self::systems::{FlyMovementSystem, FreeRotationSystem, MouseCenterLockSystem, MouseFocusUpdateSystem};
pub use self::resources::{WindowFocus};