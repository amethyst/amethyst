//! A collection of structures and functions useful across the entire amethyst project.
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

#[cfg(all(target_os = "emscripten", not(no_threading)))]
compile_error!("the cfg flag \"no_threading\" is required when building for emscripten");

pub use alga;
pub use approx;
pub use legion as ecs;
pub use nalgebra as math;
pub use num_traits as num;
pub use shrev;

use std::sync::Arc;

pub use crate::timing::*;

pub use self::{
    axis::{Axis2, Axis3},
    hidden::{Hidden, HiddenPropagate},
    named::Named,
};

/// The frame limiter module.
pub mod frame_limiter;
/// Dispatcher module.
pub mod dispatcher;
/// The geometry module.
pub mod geometry;
/// The timing module.
pub mod timing;
/// The transformation module.
pub mod transform;

mod axis;
//mod event;
mod hidden;
//mod hide_system;
mod named;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ArcThreadPool = Arc<rayon::ThreadPool>;
