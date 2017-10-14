//! # amethyst_assets
//!
//! Asset management crate. Designed with the following goals
//! in mind:
//!
//! * extensibility
//! * asynchronous & parallel using rayon and futures
//! * allow different sources

#![warn(missing_docs)]

extern crate crossbeam;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate hibitset;
extern crate parking_lot;
extern crate rayon;
extern crate specs;

pub use specs::error::BoxedErr;

pub use asset::{Asset, AssetSpec, Format};
pub use cache::Cache;
pub use error::{AssetError, LoadError, NoError};
pub use loader::{Loader, Progress, SourceId};
pub use source::{Directory, Source};
pub use storage::{AssetStorage, Handle};

mod asset;
mod cache;
mod error;
mod loader;
//mod reload;
mod source;
mod storage;
