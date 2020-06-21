//! Provides a toolbox of networking utilities, resources, components, and systems to amethyst.
//! The library is segmented into the simulation module and, eventually, various client library
//! modules. Soon, we will also provide an HTTP client library.

#[macro_use]
extern crate derive_new;

pub mod simulation;
pub use bytes::*;
