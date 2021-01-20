#![allow(clippy::default_trait_access)]
//! GPU POD data types.

use amethyst_core::math::Point3;
use amethyst_rendy::{
    pod::IntoPod,
    rendy::{
        hal::format::Format,
        mesh::{AsVertex, VertexFormat},
    },
    resources::Tint as TintComponent,
    Sprite,
};
use glsl_layout::{mat4, uvec3, vec2, vec4, Uniform};

/// `TileMapArgs`
/// ```glsl
/// uniform TileMapArgs {
///    uniform mat4 proj;
///    uniform mat4 view;
///    uniform mat4 map_coordinate_transform;
///    uniform mat4 map_transform;
/// };
/// ```
#[derive(Clone, Copy, Debug, Uniform)]
#[repr(C, align(16))]
pub struct TileMapArgs {
    /// Projection matrix
    pub proj: mat4,
    /// View matrix
    pub view: mat4,
    /// Projection matrix
    pub map_coordinate_transform: mat4,
    /// View matrix
    pub map_transform: mat4,
    /// Sprite Dimensions. Because we assume tiles are uniform for a map, we can store these here.
    pub sprite_dimensions: vec2,
}

/// Tile Vertex Data
/// ```glsl
/// vec2 dir_x;
/// vec2 dir_y;
/// vec2 pos;
/// vec2 u_offset;
/// vec2 v_offset;
/// float depth;
/// vec4 tint;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Uniform)]
#[repr(C, align(4))]
pub struct TileArgs {
    /// Upper-left coordinate of the sprite in the spritesheet
    pub u_offset: vec2,
    /// Bottom-right coordinate of the sprite in the spritesheet
    pub v_offset: vec2,
    /// Tint for this this sprite
    pub tint: vec4,
    /// Tile coordinate
    pub tile_coordinate: uvec3,
}

impl AsVertex for TileArgs {
    #[must_use]
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "u_offset"),
            (Format::Rg32Sfloat, "v_offset"),
            (Format::Rgba32Sfloat, "tint"),
            (Format::Rgb32Uint, "tile_coordinate"),
        ))
    }
}

impl TileArgs {
    #[allow(clippy::cast_precision_loss)]
    /// Extracts POD vertex data from the provided storages for a tile.
    ///
    /// # Arguments
    /// * `sprite_number` - The number index of the sprite in the sprite sheet.
    /// * `tint` - An optional `TintComponent` reference for tinting this tile, if applicable.
    /// * `tile_coordinate` - The  Point3<u32> position of this tile (in Tile Coordinate Space)
    #[must_use]
    pub fn from_data<'a>(
        sprites: &'a [Sprite],
        sprite_number: usize,
        tint: Option<&TintComponent>,
        tile_coordinate: &Point3<u32>,
    ) -> Self {
        let sprite = &sprites[sprite_number];

        Self {
            u_offset: [sprite.tex_coords.left, sprite.tex_coords.right].into(),
            v_offset: [sprite.tex_coords.top, sprite.tex_coords.bottom].into(),
            tint: tint.map_or([1.0; 4].into(), |t| t.0.into_pod()),
            tile_coordinate: [tile_coordinate.x, tile_coordinate.y, tile_coordinate.z].into(),
        }
    }
}
