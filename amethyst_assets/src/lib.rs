//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * extensibility
//! * asynchronous & parallel using rayon
//! * allow different sources

#![warn(missing_docs)]

extern crate amethyst_core;
extern crate crossbeam;
#[macro_use]
extern crate derivative;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate fnv;
extern crate hibitset;
#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate rayon;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use asset::{Asset, Format, FormatValue, SimpleFormat};
pub use cache::Cache;
pub use error::{Error, ErrorKind, Result};
pub use loader::Loader;
pub use progress::{Completion, Progress, ProgressCounter, Tracker};
pub use reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile};
pub use source::{Directory, Source};
pub use storage::{AssetStorage, Handle, Processor, WeakHandle};

mod asset;
mod cache;
mod error;
mod loader;
mod progress;
mod reload;
mod source;
mod storage;
