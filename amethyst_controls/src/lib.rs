//! Amethyst control crate.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

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

mod bundles;
mod components;
mod resources;
mod systems;
