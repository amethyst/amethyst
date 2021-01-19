//! 2D/3D Tile data structures and functionality.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![deny(clippy::all, clippy::pedantic, missing_docs)]
#![allow(dead_code, clippy::module_name_repetitions)]

mod map;
mod morton;
mod pass;

pub mod error;
pub mod iters;
pub mod pod;

use amethyst_core::math::Vector3;
pub use error::TileOutOfBoundsError;
pub use iters::{MortonRegion, Region};
pub use map::{Map, MapStorage, Tile, TileMap};
pub use morton::{MortonEncoder, MortonEncoder2D};
pub use pass::{
    DrawTiles2D, DrawTiles2DBounds, DrawTiles2DBoundsCameraCulling, DrawTiles2DBoundsDefault,
    DrawTiles2DDesc, RenderTiles2D,
};

/// Trait to provide generic access to various encoding schemas. All tile storages use this to encode their coordinates
/// and provide different spatial encoding algorithms for efficiency.
pub trait CoordinateEncoder: 'static + Clone + Default + Send + Sync {
    /// Constructor interface for `Self` which consumes the maps dimensions, which is required for some
    /// encoding types to fit within a given coordinate space.
    fn from_dimensions(dimensions: Vector3<u32>) -> Self;

    /// Encode the provided x, y and z 3-dimensional coordinates into a 1-dimensional array index.
    fn encode(&self, x: u32, y: u32, z: u32) -> Option<u32>;

    /// Decode the provided 1-dimensional array index into its associated 3-dimensional coordinates.
    fn decode(&self, morton: u32) -> Option<(u32, u32, u32)>;

    /// This function returns the actual number of elements allocated for a given dimension set and encoder.
    fn allocation_size(dimensions: Vector3<u32>) -> usize;
}

/// The most basic encoder, which strictly flattens the 3d space into 1d coordinates in a linear fashion.
/// This encoder is optimal for storage space, but not for traversal or iteration.
#[derive(Clone)]
pub struct FlatEncoder {
    dimensions: Vector3<u32>,
}
impl Default for FlatEncoder {
    #[must_use]
    fn default() -> Self {
        Self {
            dimensions: Vector3::new(0, 0, 0),
        }
    }
}
impl CoordinateEncoder for FlatEncoder {
    #[must_use]
    fn from_dimensions(dimensions: Vector3<u32>) -> Self {
        Self { dimensions }
    }

    #[inline]
    #[must_use]
    fn encode(&self, x: u32, y: u32, z: u32) -> Option<u32> {
        Some((z * self.dimensions.x * self.dimensions.y) + (y * self.dimensions.x) + x)
    }

    #[inline]
    #[must_use]
    fn decode(&self, idx: u32) -> Option<(u32, u32, u32)> {
        let z = idx / (self.dimensions.x * self.dimensions.y);
        let idx = idx - (z * self.dimensions.x * self.dimensions.y);
        let y = idx / self.dimensions.x;
        let x = idx % self.dimensions.x;

        Some((x, y, z))
    }

    #[must_use]
    fn allocation_size(dimensions: Vector3<u32>) -> usize {
        (dimensions.x * dimensions.y * dimensions.z) as usize
    }
}
