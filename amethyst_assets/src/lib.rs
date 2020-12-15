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
    loader::Loader,
    progress::{Completion, Progress, ProgressCounter, Tracker},
    reload::{HotReloadBundle, HotReloadStrategy, HotReloadSystem, Reload, SingleFile},
    source::{Directory, Source},
    storage::{AssetProcessorSystemBundle, AssetStorage, Handle, ProcessingState, WeakHandle, build_asset_processor_system},
};

mod asset;
mod cache;
mod dyn_format;
mod error;
mod formats;
mod loader;
// FIXME: new prefab system
// mod prefab;
mod progress;
mod reload;
mod source;
mod storage;

// used in macros. Private API otherwise.
#[doc(hidden)]
pub use {erased_serde, inventory, lazy_static};

#[doc(hidden)]
pub use crate::dyn_format::{DeserializeFn, Registry};
