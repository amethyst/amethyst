//! Renderer error types.

use amethyst_core::math::{Point3, Vector3};
use err_derive::Error;

/// Tile is out of bounds.
#[derive(Debug, Error)]
#[error(
    display = "Requested coordinate is outside map dimensions: '{:?}', max dimensions: '{:?}'",
    point_dimensions,
    max_dimensions
)]
pub struct TileOutOfBoundsError {
    /// Calculated Point Dimension.
    pub point_dimensions: Point3<i32>,
    /// Map dimensions.
    pub max_dimensions: Vector3<u32>,
}
