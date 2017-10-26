
pub extern crate cgmath;

#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate hibitset;
extern crate rayon;
#[macro_use]
extern crate serde;
extern crate shred;
extern crate specs;

//#[cfg(test)]
//extern crate quickcheck;

pub use bundle::{ECSBundle, Error, ErrorKind, Result};
pub use timing::*;
pub use transform::*;

use std::sync::Arc;

pub mod bundle;
pub mod orientation;
pub mod transform;
pub mod timing;
pub mod frame_limiter;

/// A rayon thread pool wrapped in an `Arc`. This should be used as resource in `World`.
pub type ThreadPool = Arc<rayon::ThreadPool>;
