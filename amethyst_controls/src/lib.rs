//! Amethyst control crate.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
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
        ArcBallRotationSystem, CursorHideSystem, CursorHideSystemDesc, FlyMovementSystem,
        FlyMovementSystemDesc, FreeRotationSystem, FreeRotationSystemDesc, MouseFocusUpdateSystem,
        MouseFocusUpdateSystemDesc,
    },
};

mod bundles;
mod components;
mod resources;
mod systems;
