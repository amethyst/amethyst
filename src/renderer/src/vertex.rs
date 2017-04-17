//! Built-in vertex formats.

use gfx;
use gfx::format::Format;
use gfx::pso::buffer::Structure;
use gfx::traits::Pod;
use std::fmt::Debug;

/// Handle to a vertex attribute.
pub type Attribute = gfx::pso::buffer::Element<Format>;

/// Trait implemented by all valid vertex formats.
pub trait VertexFormat: Debug + Pod + Structure<Format> + Sized {
    /// Returns a list of all attributes specified in the vertex.
    fn attributes() -> Vec<Attribute>;
    /// Returns the size of a single vertex in bytes.
    fn size() -> usize {
        use std::mem;
        mem::size_of::<Self>()
    }
}

/// Vertex format with position and RGBA8 color attributes.
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosColor {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// RGBA color value of the vertex.
    pub a_color: [f32; 4],
}

impl VertexFormat for PosColor {
    #[inline]
    fn attributes() -> Vec<Attribute> {
        vec![Self::query("a_position").unwrap(), Self::query("a_color").unwrap()]
    }
}

/// Vertex format with position and UV texture coordinate attributes.
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosTex {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// UV texture coordinates used by the vertex.
    pub a_tex_coord: [f32; 2],
}

impl VertexFormat for PosTex {
    #[inline]
    fn attributes() -> Vec<Attribute> {
        vec![Self::query("a_position").unwrap(), Self::query("a_tex_coord").unwrap()]
    }
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosNormTex {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// Normal vector of the vertex.
    pub a_normal: [f32; 4],
    /// UV texture coordinates used by the vertex.
    pub a_tex_coord: [f32; 2],
}

impl VertexFormat for PosNormTex {
    #[inline]
    fn attributes() -> Vec<Attribute> {
        vec![Self::query("a_position").unwrap(),
             Self::query("a_normal").unwrap(),
             Self::query("a_tex_coord").unwrap()]
    }
}
