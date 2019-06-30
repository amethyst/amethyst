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
};
#[cfg(feature = "experimental-assets")]
pub use crate::{
    loader_new::{
        create_asset_type, AssetHandle, AssetUuid, DefaultLoader as NewDefaultLoader,
        GenericHandle, Handle as NewHandle, LoadStatus, Loader as NewLoader,
    },
    processor::{ProcessingQueue, ProcessingState as NewProcessingState},
    simple_importer::{SimpleImporter, SourceFileImporter},
    storage_new::AssetStorage as NewAssetStorage,
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

#[cfg(feature = "experimental-assets")]
mod loader_new;
#[cfg(feature = "experimental-assets")]
mod processor;
#[cfg(feature = "experimental-assets")]
mod simple_importer;
#[cfg(feature = "experimental-assets")]
mod storage_new;

/// Registers an importer for the new experimental asset system
#[cfg(not(feature = "experimental-assets"))]
#[macro_export]
macro_rules! register_importer {
    ($ext:literal, $format:ty) => {};
    ($krate:ident; $ext:literal, $format:ty) => {};
}

/// Registers an intermediate -> asset type for the new experimental asset system
#[cfg(not(feature = "experimental-assets"))]
#[macro_export]
macro_rules! register_asset_type {
    ($intermediate:ty => $asset:ty) => {};
    ($krate:ident; $intermediate:ty => $asset:ty) => {};
}

// used in macros. Private API otherwise.
#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};
