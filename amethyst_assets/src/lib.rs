//! # amethyst_assets
//!
//! Asset management crate.
//! Designed with the following goals in mind:
//!
//! * extensibility
//! * asynchronous & parallel using rayon
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

pub use asset::{Asset, Format};
pub use error::AssetError;
pub use loader::{Loader, Progress};
pub use source::{Directory, Source};
pub use storage::{AssetStorage, Handle, Processor};

mod asset;
//mod cache;
mod error;
mod loader;
//mod reload;
mod source;
mod storage;
