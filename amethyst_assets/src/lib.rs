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
    helper::AssetLoaderSystemData,
    loader::Loader,
    prefab::{
        AssetPrefab, Prefab, PrefabData, PrefabLoader, PrefabLoaderSystem, PrefabLoaderSystemDesc,
    },
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetStorage, Handle, ProcessingState, Processor, WeakHandle},
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
#[cfg(feature = "experimental-assets")]
mod handle_new;
#[cfg(feature = "experimental-assets")]
/// Experimental module for testing new asset loading features
pub mod experimental {
    pub use crate::{
        loader_new::{
            create_asset_type, AssetUuid, DefaultLoader,
            LoadStatus, Loader,
        },
        handle_new::{
            GenericHandle, Handle, AssetHandle, 
        },
        processor::{
            ProcessingQueue, ProcessingState, Processor,
        },
        simple_importer::{SimpleImporter, SourceFileImporter},
        storage_new::AssetStorage,
    };
}

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
    ($intermediate:ty => $asset:ty; $system:ty) => {};
    ($krate:ident; $intermediate:ty => $asset:ty; $system:ty) => {};
}

// used in macros. Private API otherwise.
#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};
