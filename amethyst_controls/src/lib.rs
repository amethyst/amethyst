//! Amethyst control crate.

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub use self::{
    bundles::{ArcBallControlBundle, FlyControlBundle},
    components::{ArcBallControlTag, ControlTagPrefab, FlyControlTag},
    resources::{HideCursor, WindowFocus},
    systems::{
        ArcBallRotationSystem, CursorHideSystem, FlyMovementSystem, FreeRotationSystem,
        MouseFocusUpdateSystem,
    },
};

use amethyst_core;

#[macro_use]
extern crate derive_new;

mod bundles;
mod components;
mod resources;
mod systems;
