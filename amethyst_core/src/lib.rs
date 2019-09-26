//! A collection of structures and functions useful across the entire amethyst project.
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

#[cfg(all(target_os = "emscripten", not(no_threading)))]
compile_error!("the cfg flag \"no_threading\" is required when building for emscripten");

#[macro_use]
extern crate getset;
#[macro_use]
extern crate derive_new;

pub use alga;
pub use approx;
pub use nalgebra as math;
pub use num_traits as num;
pub use specs as ecs;
pub use specs::{shred, shrev};

use rayon;

use std::sync::Arc;

pub use crate::{
    bundle::SystemBundle,
    event::EventReader,
    system_ext::{Pausable, SystemExt},
    timing::*,
    transform::*,
};

pub use self::{
    axis::{Axis2, Axis3},
    hidden::{Hidden, HiddenPropagate},
    hide_system::{HideHierarchySystem, HideHierarchySystemDesc},
    named::{Named, WithNamed},
    system_desc::{RunNowDesc, SystemDesc},
};

pub mod bundle;
pub mod deferred_dispatcher_operation;
pub mod frame_limiter;
pub mod geometry;
pub mod timing;
pub mod transform;

mod axis;
mod event;
mod hidden;
mod hide_system;
mod named;
mod system_desc;
mod system_ext;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ArcThreadPool = Arc<rayon::ThreadPool>;
