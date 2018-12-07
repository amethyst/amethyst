//! Amethyst control crate.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]
use amethyst_core;

#[macro_use]
extern crate serde;

mod bundles;
mod components;
mod resources;
mod systems;

pub use self::{
    bundles::{ArcBallControlBundle, FlyControlBundle},
    components::{ArcBallControlTag, ControlTagPrefab, FlyControlTag},
    resources::{HideCursor, WindowFocus},
    systems::{
        ArcBallRotationSystem, CursorHideSystem, FlyMovementSystem, FreeRotationSystem,
        MouseFocusUpdateSystem,
    },
};
