//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * extensibility
//! * asynchronous & parallel using rayon
//! * allow different sources

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))] // complex project

extern crate amethyst_core;
extern crate crossbeam;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate hibitset;
#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate rayon;
extern crate ron;
#[macro_use]
extern crate serde;
#[cfg(feature = "json")]
extern crate serde_json;
extern crate shred;
#[macro_use]
extern crate shred_derive;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use asset::{Asset, Format, FormatValue, SimpleFormat};
pub use cache::Cache;
pub use error::{Error, ErrorKind, Result, ResultExt};
#[cfg(feature = "json")]
pub use formats::JsonFormat;
pub use formats::RonFormat;
pub use loader::Loader;
pub use prefab::{AssetPrefab, Prefab, PrefabData, PrefabError, PrefabLoader, PrefabLoaderSystem};
pub use progress::{Completion, Progress, ProgressCounter, Tracker};
pub use reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile};
pub use source::{Directory, Source};
pub use storage::{AssetStorage, Handle, ProcessingState, Processor, WeakHandle};

mod asset;
mod cache;
mod error;
mod formats;
mod loader;
mod prefab;
mod progress;
mod reload;
mod source;
mod storage;
