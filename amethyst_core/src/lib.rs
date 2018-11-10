//! A collection of structures and functions useful across the entire amethyst project.

#![warn(missing_docs)]

pub extern crate approx;
pub extern crate nalgebra;
pub extern crate shred;
pub extern crate shrev;
pub extern crate specs;

#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate hibitset;
extern crate log;
extern crate rayon;
#[macro_use]
extern crate serde;
extern crate specs_hierarchy;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

#[cfg(all(target_os = "emscripten", not(no_threading)))]
compile_error!("the cfg flag \"no_threading\" is required when building for emscripten");

use std::sync::Arc;

pub use {
    bundle::{Error, ErrorKind, Result, SystemBundle},
    event::EventReader,
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
pub mod timing;
pub mod transform;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ArcThreadPool = Arc<rayon::ThreadPool>;
