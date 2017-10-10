//! # amethyst_assets
//!
//! Asset management crate. Designed with the following goals
//! in mind:
//!
//! * extensibility
//! * asynchronous & parallel using rayon and futures
//! * allow different stores

#![warn(missing_docs)]

extern crate fnv;
extern crate futures;
extern crate parking_lot;
extern crate rayon;
extern crate specs;

pub use specs::error::BoxedErr;

pub use asset::{Asset, AssetSpec, AssetUpdates, Context, Format};
pub use cache::Cache;
pub use error::{AssetError, LoadError, NoError, SharedAssetError};
pub use loader::{load_asset, Loader, SpawnedFuture, StoreId};
pub use simple::SimpleContext;
pub use store::{Directory, Store};

mod asset;
mod cache;
mod error;
mod loader;
mod simple;
mod store;
mod stream;
