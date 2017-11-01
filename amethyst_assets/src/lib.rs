//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * extensibility
//! * asynchronous & parallel using rayon
//! * allow different sources
//!
//! # Overview
//!
//! The most important type of this crate is the `AssetStorage`.
//! It's the place where all the assets are located; it only gives you
//! `Handle`s to them which you can use to get a reference to the actual asset.
//! The `Loader` is responsible for loading asset data, an intermediate format in asset loading.
//! Asset data will then be pushed to a queue which the `AssetStorage` has access to.
//! After that, we of course need to transform asset data into an asset, which happens
//! by calling `AssetStorage::process`. After the data is processed, the handle will
//! allow you to access the stored asset.
//!

#![warn(missing_docs)]

extern crate amethyst_core;
extern crate crossbeam;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate error_chain;
extern crate fnv;
extern crate hibitset;
extern crate parking_lot;
extern crate rayon;
extern crate ron;
#[macro_use]
extern crate serde;
extern crate specs;

pub use asset::{Asset, Format, FormatValue, SimpleFormat};
pub use cache::Cache;
pub use dyn::AssetsDeserializer;
pub use error::{Error, ErrorKind, Result, ResultExt};
pub use loader::Loader;
pub use progress::{Completion, Progress, ProgressCounter, Tracker};
pub use reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile};
pub use source::{Directory, Source};
pub use storage::{AssetStorage, Handle, Processor, WeakHandle};

mod asset;
mod cache;
mod dyn;
mod error;
mod loader;
mod progress;
mod reload;
mod source;
mod storage;
