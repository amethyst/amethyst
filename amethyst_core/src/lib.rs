#[macro_use]
pub extern crate cgmath;

#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate hibitset;
extern crate rayon;
#[macro_use]
extern crate serde;
extern crate shred;
extern crate shrev;
extern crate specs;
extern crate winit;

//#[cfg(test)]
//extern crate quickcheck;

pub use bundle::{ECSBundle, Error, ErrorKind, Result};
pub use events_pump::EventsPump;
pub use timing::*;
pub use transform::*;

use std::sync::Arc;

pub mod bundle;
pub mod events_pump;
pub mod orientation;
pub mod transform;
pub mod timing;
pub mod frame_limiter;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ThreadPool = Arc<rayon::ThreadPool>;
