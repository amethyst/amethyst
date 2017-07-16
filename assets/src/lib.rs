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

pub use specs::common::BoxedErr;

pub use asset::{Asset, AssetSpec, Cache, Format};
pub use error::{AssetError, LoadError, NoError};
pub use loader::{AssetFuture, Loader, load_asset, load_asset_future};
pub use store::{Allocator, Directory, Store, StoreId};

mod asset;
mod error;
mod loader;
mod store;
