//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * Extensibility
//! * Asynchronous & Parallel using Rayon
//! * Allow different sources

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

pub use rayon::ThreadPool;

#[cfg(feature = "json")]
pub use crate::formats::JsonFormat;
pub use crate::{
    asset::{Asset, Format, FormatValue, ProcessableAsset, SerializableFormat},
    cache::Cache,
    dyn_format::FormatRegisteredData,
    formats::RonFormat,
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{build_hot_reload_system, HotReloadBundle, HotReloadStrategy, Reload, SingleFile},
    source::{Directory, Source},
};

mod asset;
mod bundle;
mod cache;
mod dyn_format;
pub mod error;
mod formats;
mod loader;
pub mod prefab;
mod processor;
mod progress;
mod reload;
mod simple_importer;
mod source;
mod storage_new;
/// Experimental module for testing new asset loading features
pub mod experimental {
    pub use atelier_core::TypeUuidDynamic;
    pub use atelier_loader::{
        asset_uuid,
        handle::{AssetHandle, GenericHandle, Handle},
    };

    pub use crate::{
        bundle::LoaderBundle,
        loader::{create_asset_type, AssetUuid, DefaultLoader, LoadStatus, Loader},
        processor::{ProcessingQueue, ProcessingState},
        simple_importer::{SimpleImporter, SourceFileImporter},
        storage_new::AssetStorage,
    };
}
pub use atelier_loader::{
    handle::{AssetHandle, GenericHandle, Handle, WeakHandle},
    storage::LoadHandle,
};
pub use bundle::{start_asset_daemon, LoaderBundle};
pub use loader::{create_asset_type, AssetUuid, DefaultLoader, LoadStatus, Loader};
pub use processor::{AddToDispatcher, DefaultProcessor, ProcessingQueue, ProcessingState};
pub use storage_new::AssetStorage;
// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};

#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
