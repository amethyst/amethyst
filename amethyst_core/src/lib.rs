#[macro_use]
pub extern crate cgmath;
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

pub use self::named::{Named, WithNamed};
pub use bundle::{Error, ErrorKind, Result, SystemBundle};
pub use orientation::Orientation;
use std::sync::Arc;
pub use timing::*;
pub use transform::*;

pub mod bundle;
pub mod frame_limiter;
mod named;
mod orientation;
pub mod timing;
pub mod transform;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ThreadPool = Arc<rayon::ThreadPool>;
