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

#[cfg(feature = "json")]
pub use formats::JsonFormat;
pub use {
    asset::{Asset, Format, FormatValue, SimpleFormat},
    cache::Cache,
    error::{Error, ErrorKind, Result, ResultExt},
    formats::RonFormat,
    helper::AssetLoaderSystemData,
    loader::Loader,
    prefab::{AssetPrefab, Prefab, PrefabData, PrefabError, PrefabLoader, PrefabLoaderSystem},
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetStorage, Handle, ProcessingState, Processor, WeakHandle},
};

mod asset;
mod cache;
mod error;
mod formats;
mod helper;
mod loader;
mod prefab;
mod progress;
mod reload;
mod source;
mod storage;
