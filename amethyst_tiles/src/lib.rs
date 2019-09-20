//! 2D/3D Tile data structures and functionality.
//!

#![deny(clippy::all, clippy::pedantic, missing_docs)]
#![allow(dead_code, clippy::module_name_repetitions)]

mod map;
mod morton;
mod pass;

pub mod iters;
pub mod pod;

pub use iters::{MortonRegion, Region};
pub use map::{Map, MapStorage, Tile, TileMap};
pub use morton::{MortonEncoder, MortonEncoder2D};
pub use pass::{
    DrawTiles2D, DrawTiles2DBounds, DrawTiles2DBoundsDefault, DrawTiles2DDesc, RenderTiles2D,
};

/// Trait to provide generic access to various encoding schemas. All tile storages use this to encode their coordinates
/// and provide different spatial encoding algorithms for efficiency.
pub trait CoordinateEncoder: 'static + Clone + Default + Send + Sync {
    /// Constructor interface for `Self` which consumes the maps dimensions, which is required for some
    /// encoding types to fit within a given coordinate space.
    fn from_dimensions(x: u32, y: u32, z: u32) -> Self;

    /// Encode the provided x, y and z 3-dimensional coordinates into a 1-dimensional array index.
    fn encode(&self, x: u32, y: u32, z: u32) -> Option<u32>;

    /// Decode the provided 1-dimensional array index into its associated 3-dimensional coordinates.
    fn decode(&self, morton: u32) -> Option<(u32, u32, u32)>;
}
