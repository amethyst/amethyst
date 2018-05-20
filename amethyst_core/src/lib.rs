//! This crate contains the `SystemBundle` abstraction, and functionality for dealing with
//! transforms.
#[macro_use]
pub extern crate cgmath;
pub extern crate shred;
pub extern crate shrev;
pub extern crate specs;

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate fnv;
extern crate hibitset;
#[macro_use]
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

//#[cfg(test)]
//extern crate quickcheck;

pub use bundle::SystemBundle;
pub use error::{Error, ErrorKind, Result};
pub use orientation::Orientation;
pub use timing::*;
pub use transform::*;

use std::sync::Arc;

pub mod bundle;
mod error;
pub mod frame_limiter;
mod orientation;
pub mod timing;
pub mod transform;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ThreadPool = Arc<rayon::ThreadPool>;
