//! A collection of structures and functions useful across the entire amethyst project.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

pub use approx;
pub use nalgebra;
pub use shred;
pub use shrev;
pub use specs;

#[macro_use]
extern crate error_chain;
use rayon;
#[macro_use]
extern crate serde;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

#[cfg(all(target_os = "emscripten", not(no_threading)))]
compile_error!("the cfg flag \"no_threading\" is required when building for emscripten");

use std::sync::Arc;

pub use crate::{
    bundle::{Error, ErrorKind, Result, SystemBundle},
    event::EventReader,
    system_ext::{Pausable, SystemExt},
    timing::*,
    transform::*,
};

pub use self::{
    axis::{Axis2, Axis3},
    named::{Named, WithNamed},
};

mod axis;
pub mod bundle;
mod event;
pub mod frame_limiter;
mod named;
mod system_ext;
pub mod timing;
pub mod transform;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ArcThreadPool = Arc<rayon::ThreadPool>;
