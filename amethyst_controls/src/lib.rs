//! Amethyst control crate.

#![warn(missing_docs)]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_input;
extern crate amethyst_renderer;
#[macro_use]
extern crate serde;
extern crate winit;

#[cfg(feature = "profiler")]
extern crate thread_profiler;

mod bundles;
mod components;
mod resources;
mod systems;

pub use self::bundles::{ArcBallControlBundle, FlyControlBundle};
pub use self::components::{ArcBallControlTag, ControlTagPrefab, FlyControlTag};
pub use self::resources::{HideCursor, WindowFocus};
pub use self::systems::{
    ArcBallRotationSystem, CursorHideSystem, FlyMovementSystem, FreeRotationSystem,
    MouseFocusUpdateSystem,
};
