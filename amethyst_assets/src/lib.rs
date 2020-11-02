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
    // loader::Loader,
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{build_hot_reload_system, HotReloadBundle, HotReloadStrategy, Reload, SingleFile},
    source::{Directory, Source},
    // storage::{AssetProcessorSystemBundle, AssetStorage, Handle, ProcessingState, WeakHandle},
};

pub use rayon::ThreadPool;

mod asset;
mod cache;
mod dyn_format;
pub mod error;
mod formats;
// mod helper;
// mod loader;
// mod prefab;
mod progress;
mod reload;
mod source;
// mod storage;

mod bundle_new;
mod loader_new;
mod processor;
mod simple_importer;
mod storage_new;
/// Experimental module for testing new asset loading features
pub mod experimental {
    pub use crate::{
        bundle_new::LoaderBundle,
        loader_new::{create_asset_type, AssetUuid, DefaultLoader, LoadStatus, Loader},
        processor::{ProcessingQueue, ProcessingState},
        simple_importer::{SimpleImporter, SourceFileImporter},
        storage_new::AssetStorage,
    };
    pub use atelier_core::TypeUuidDynamic;
    pub use atelier_loader::asset_uuid;
    pub use atelier_loader::handle::{AssetHandle, GenericHandle, Handle};
}
pub use atelier_loader::{
    handle::{AssetHandle, GenericHandle, Handle, WeakHandle},
    storage::LoadHandle,
};
pub use bundle_new::{start_asset_daemon, LoaderBundle};
pub use loader_new::{create_asset_type, AssetUuid, DefaultLoader, LoadStatus, Loader};
pub use processor::{AddToDispatcher, DefaultProcessor, ProcessingQueue, ProcessingState};

pub use storage_new::AssetStorage;
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};
