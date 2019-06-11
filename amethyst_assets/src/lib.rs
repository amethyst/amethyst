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
    asset::{Asset, Format, FormatValue, ProcessableAsset},
    cache::Cache,
    dyn_format::FormatRegisteredData,
    formats::RonFormat,
    helper::AssetLoaderSystemData,
    loader::Loader,
    prefab::{AssetPrefab, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem},
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetStorage, Handle, ProcessingState, Processor, WeakHandle},
    loader_new::{create_asset_type, TypeUuid, AssetUuid},
    simple_importer::{SourceFileImporter, SimpleImporter},
};

pub use rayon::ThreadPool;

mod asset;
mod cache;
mod dyn_format;
mod error;
mod formats;
mod helper;
mod loader;
mod prefab;
mod progress;
mod reload;
mod source;
mod storage;
mod storage_new;
mod loader_new;
mod processor;
#[cfg(feature = "importers")]
mod simple_importer;

// used in macros. Private API otherwise.
#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};
