//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * Extensibility
//! * Asynchronous & Parallel using Rayon
//! * Allow different sources

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[cfg(feature = "json")]
pub use crate::formats::JsonFormat;
pub use crate::{
    asset::{Asset, Format, FormatValue, ProcessableAsset, SerializableFormat},
    cache::Cache,
    dyn_format::FormatRegisteredData,
    formats::RonFormat,
    loader::Loader,
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{build_hot_reload_system, HotReloadBundle, HotReloadStrategy, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetProcessorSystemBundle, AssetStorage, Handle, ProcessingState, WeakHandle},
};

pub use rayon::ThreadPool;

mod asset;
mod cache;
mod dyn_format;
mod error;
mod formats;
// mod helper;
mod loader;
// mod prefab;
mod progress;
mod reload;
mod source;
mod storage;

// used in macros. Private API otherwise.
#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};
