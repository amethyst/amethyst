//! A collection of structures and functions useful across the entire amethyst project.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(all(target_os = "emscripten", not(no_threading)))]
compile_error!("the cfg flag \"no_threading\" is required when building for emscripten");

#[macro_use]
extern crate alga_derive;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate getset;
#[macro_use]
extern crate derive_new;

pub use alga;
pub use approx;
pub use nalgebra as math;
pub use num_traits as num;
pub use shred;
pub use shrev;
pub use specs as ecs;

use rayon;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

use std::sync::Arc;

pub use crate::{
    bundle::SystemBundle,
    event::EventReader,
    float::Float,
    system_ext::{Pausable, SystemExt},
    timing::*,
    transform::*,
};

pub use self::{
    axis::{Axis2, Axis3},
    named::{Named, WithNamed},
};

pub mod bundle;
pub mod frame_limiter;
pub mod timing;
pub mod transform;

mod axis;
mod event;
mod float;
mod named;
mod system_ext;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ArcThreadPool = Arc<rayon::ThreadPool>;
