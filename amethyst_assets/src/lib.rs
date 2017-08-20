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
#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate rayon;
extern crate specs;

pub use specs::common::BoxedErr;

pub use asset::{Asset, AssetFuture, AssetSpec, Cache, Context, Format};
pub use error::{AssetError, LoadError, NoError};
pub use loader::{SpawnedFuture, Loader, load_asset};
pub use simple::{AssetPtr, SimpleContext};
pub use store::{Allocator, Directory, Store, StoreId};

mod asset;
mod error;
mod loader;
mod simple;
mod store;
