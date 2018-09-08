//! Amethyst control crate.
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
pub use self::components::{ArcBallControlTag, ControlTagPrefab, FlyControlTag, XYControlTag};
pub use self::resources::WindowFocus;
pub use self::systems::{
    ArcBallMovementSystem, CursorHideSystem, FlyMovementSystem, FreeRotationSystem,
    MouseFocusUpdateSystem, XYCameraSystem,
};
