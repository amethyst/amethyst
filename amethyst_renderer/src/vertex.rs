//! Built-in vertex formats.

use gfx;
use gfx::format::Format;
use gfx::pso::buffer::Structure;
use gfx::traits::Pod;
use std::any::Any;

/// Handle to a vertex attribute.
pub type Attribute = gfx::pso::buffer::Element<Format>;

/// Type for position attribute of vertex
pub enum Position {}

/// Type for color attribute of vertex
pub enum Color {}

/// Type for texture coord attribute of vertex
pub enum TextureCoord {}

/// Type for texture coord attribute of vertex
pub enum Normal {}

/// Type for tangent attribute of vertex
pub enum Tangent {}

/// Trait for mapping attribute type -> name
pub trait AttributeNames {
    /// Get name for specified attribute type
    fn name<A: Any>() -> &'static str;
}

/// Trait implemented by all valid vertex formats.
pub trait VertexFormat: Pod + Structure<Format> + Sized + Send + Sync {
    /// Container for attributes of this format
    type Attributes: AsRef<[Attribute]>;

    /// Container for name+attribute pairs of this format
    type NamedAttributes: AsRef<[(&'static str, Attribute)]>;

    /// Returns a list of all attributes specified in the vertex.
    fn attributes() -> Self::Attributes;

    /// Returns a list of all name+attribute pairs specified in the vertex.
    /// The caller provides attribute type -> Name mapping
    fn named_attributes<N: AttributeNames>() -> Self::NamedAttributes;

    /// Returns the size of a single vertex in bytes.
    #[inline]
    fn size() -> usize {
        use std::mem;
        mem::size_of::<Self>()
    }

    /// Returns attribute of vertex by type
    #[inline]
    fn attribute<F>() -> Attribute
    where
        Self: WithField<F>,
    {
        <Self as WithField<F>>::field_attribute()
    }
}

/// Trait implemented by all valid vertex formats for each field
pub trait WithField<F>: VertexFormat {
    /// Query individual attribute of the field for this format
    fn field_attribute() -> Attribute;
}

/// Vertex format with position and RGBA8 color attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosColor {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// RGBA color value of the vertex.
    pub a_color: [f32; 4],
}

impl VertexFormat for PosColor {
    type Attributes = [Attribute; 2];
    type NamedAttributes = [(&'static str, Attribute); 2];

    #[inline]
    fn attributes() -> Self::Attributes {
        [
            Self::query("a_position").unwrap(),
            Self::query("a_color").unwrap(),
        ]
    }

    #[inline]
    fn named_attributes<N: AttributeNames>() -> Self::NamedAttributes {
        [
            (N::name::<Position>(), Self::query("a_position").unwrap()),
            (N::name::<Color>(), Self::query("a_color").unwrap()),
        ]
    }
}

impl WithField<Position> for PosColor {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_position").unwrap()
    }
}

impl WithField<Color> for PosColor {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_color").unwrap()
    }
}

/// Vertex format with position and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosTex {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// UV texture coordinates used by the vertex.
    pub a_tex_coord: [f32; 2],
}

impl VertexFormat for PosTex {
    type Attributes = [Attribute; 2];
    type NamedAttributes = [(&'static str, Attribute); 2];

    #[inline]
    fn attributes() -> Self::Attributes {
        [
            Self::query("a_position").unwrap(),
            Self::query("a_tex_coord").unwrap(),
        ]
    }

    #[inline]
    fn named_attributes<N: AttributeNames>() -> Self::NamedAttributes {
        [
            (N::name::<Position>(), Self::query("a_position").unwrap()),
            (
                N::name::<TextureCoord>(),
                Self::query("a_tex_coord").unwrap(),
            ),
        ]
    }
}

impl WithField<Position> for PosTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_position").unwrap()
    }
}

impl WithField<TextureCoord> for PosTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_tex_coord").unwrap()
    }
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosNormTex {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// Normal vector of the vertex.
    pub a_normal: [f32; 3],
    /// UV texture coordinates used by the vertex.
    pub a_tex_coord: [f32; 2],
}

impl VertexFormat for PosNormTex {
    type Attributes = [Attribute; 3];
    type NamedAttributes = [(&'static str, Attribute); 3];

    #[inline]
    fn attributes() -> Self::Attributes {
        [
            Self::query("a_position").unwrap(),
            Self::query("a_normal").unwrap(),
            Self::query("a_tex_coord").unwrap(),
        ]
    }

    #[inline]
    fn named_attributes<N: AttributeNames>() -> Self::NamedAttributes {
        [
            (N::name::<Position>(), Self::query("a_position").unwrap()),
            (N::name::<Normal>(), Self::query("a_normal").unwrap()),
            (
                N::name::<TextureCoord>(),
                Self::query("a_tex_coord").unwrap(),
            ),
        ]
    }
}

impl WithField<Position> for PosNormTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_position").unwrap()
    }
}

impl WithField<Normal> for PosNormTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_normal").unwrap()
    }
}

impl WithField<TextureCoord> for PosNormTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_tex_coord").unwrap()
    }
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, VertexData)]
pub struct PosNormTangTex {
    /// Position of the vertex in 3D space.
    pub a_position: [f32; 3],
    /// Normal vector of the vertex.
    pub a_normal: [f32; 3],
    /// Tangent vector of the vertex.
    pub a_tangent: [f32; 3],
    /// UV texture coordinates used by the vertex.
    pub a_tex_coord: [f32; 2],
}

impl VertexFormat for PosNormTangTex {
    type Attributes = [Attribute; 4];
    type NamedAttributes = [(&'static str, Attribute); 4];
    #[inline]
    fn attributes() -> Self::Attributes {
        [
            Self::query("a_position").unwrap(),
            Self::query("a_normal").unwrap(),
            Self::query("a_tangent").unwrap(),
            Self::query("a_tex_coord").unwrap(),
        ]
    }
    #[inline]
    fn named_attributes<N: AttributeNames>() -> Self::NamedAttributes {
        [
            (N::name::<Position>(), Self::query("a_position").unwrap()),
            (N::name::<Normal>(), Self::query("a_normal").unwrap()),
            (N::name::<Tangent>(), Self::query("a_tangent").unwrap()),
            (
                N::name::<TextureCoord>(),
                Self::query("a_tex_coord").unwrap(),
            ),
        ]
    }
}

impl WithField<Position> for PosNormTangTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_position").unwrap()
    }
}

impl WithField<Normal> for PosNormTangTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_normal").unwrap()
    }
}

impl WithField<Tangent> for PosNormTangTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_tangent").unwrap()
    }
}

impl WithField<TextureCoord> for PosNormTangTex {
    #[inline]
    fn field_attribute() -> Attribute {
        Self::query("a_tex_coord").unwrap()
    }
}
