//! Built-in vertex formats.

use gfx::{
    format::{ChannelType, Format, SurfaceType},
    pso::buffer::Element,
    traits::Pod,
};
use serde::{Deserialize, Serialize};

use amethyst_core::nalgebra::{Vector2, Vector3};

/// Format for vertex attribute
pub type AttributeFormat = Element<Format>;

/// Slice of attributes
pub type Attributes<'a> = &'a [(&'a str, AttributeFormat)];

/// Trait for vertex attributes to implement
pub trait Attribute {
    /// Name of the attribute
    /// It is used to bind to the attributes in shaders
    const NAME: &'static str;

    /// Format of the attribute defines arity and type
    const FORMAT: Format;

    /// Size of the attribue
    const SIZE: u32; // Has to be equal to `std::mem::size_of::<Self::Repr>() as u32`

    /// Representation of the attribute
    /// usually it is `[f32; N]`
    type Repr: Pod + Send + Sync;
}

/// Type for position attribute of vertex
#[derive(Clone, Debug)]
pub enum Position {}
impl Attribute for Position {
    const NAME: &'static str = "position";
    const FORMAT: Format = Format(SurfaceType::R32_G32_B32, ChannelType::Float);
    const SIZE: u32 = 12;
    type Repr = [f32; 3];
}

/// Type for color attribute of vertex
#[derive(Clone, Debug)]
pub enum Color {}
impl Attribute for Color {
    const NAME: &'static str = "color";
    const FORMAT: Format = Format(SurfaceType::R32_G32_B32_A32, ChannelType::Float);
    const SIZE: u32 = 16;
    type Repr = [f32; 4];
}

/// Type for texture coord attribute of vertex
#[derive(Clone, Debug)]
pub enum TexCoord {}
impl Attribute for TexCoord {
    const NAME: &'static str = "tex_coord";
    const FORMAT: Format = Format(SurfaceType::R32_G32, ChannelType::Float);
    const SIZE: u32 = 8;
    type Repr = [f32; 2];
}

/// Type for texture coord attribute of vertex
#[derive(Clone, Debug)]
pub enum Normal {}
impl Attribute for Normal {
    const NAME: &'static str = "normal";
    const FORMAT: Format = Format(SurfaceType::R32_G32_B32, ChannelType::Float);
    const SIZE: u32 = 12;
    type Repr = [f32; 3];
}

/// Type for tangent attribute of vertex
#[derive(Clone, Debug)]
pub enum Tangent {}
impl Attribute for Tangent {
    const NAME: &'static str = "tangent";
    const FORMAT: Format = Format(SurfaceType::R32_G32_B32, ChannelType::Float);
    const SIZE: u32 = 12;
    type Repr = [f32; 3];
}

/// Trait implemented by all valid vertex formats.
pub trait VertexFormat: Pod + Sized + Send + Sync {
    /// List of all attributes formats with name and offset.
    const ATTRIBUTES: Attributes<'static>;

    /// Returns the size of a single vertex in bytes.
    #[inline]
    fn size() -> usize {
        use std::mem;
        mem::size_of::<Self>()
    }

    /// Returns attribute of vertex by type
    #[inline]
    fn attribute<F>() -> AttributeFormat
    where
        F: Attribute,
        Self: With<F>,
    {
        <Self as With<F>>::FORMAT
    }
}

/// Trait implemented by all valid vertex formats for each field
pub trait With<F: Attribute>: VertexFormat {
    /// Individual format of the attribute for this vertex format
    const FORMAT: AttributeFormat;
}

/// Vertex format for attributes in separate buffers
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Separate<T: Attribute>(pub T::Repr);
unsafe impl<T> Pod for Separate<T> where T: Attribute {}

impl<T> Separate<T>
where
    T: Attribute,
{
    /// Create a new Separate vertex attribute
    pub fn new(data: T::Repr) -> Self {
        Separate(data)
    }
}

impl<T> VertexFormat for Separate<T>
where
    T: Attribute,
{
    const ATTRIBUTES: Attributes<'static> = &[(
        T::NAME,
        Element {
            offset: 0,
            format: T::FORMAT,
        },
    )];
}

impl<T> With<T> for Separate<T>
where
    T: Attribute,
{
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: T::FORMAT,
    };
}

/// Vertex format with position and RGBA8 color attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PosColor {
    /// Position of the vertex in 3D space.
    pub position: Vector3<f32>,
    /// RGBA color value of the vertex.
    pub color: [f32; 4],
}

unsafe impl Pod for PosColor {}

impl VertexFormat for PosColor {
    const ATTRIBUTES: Attributes<'static> = &[
        (Position::NAME, <Self as With<Position>>::FORMAT),
        (Color::NAME, <Self as With<Color>>::FORMAT),
    ];
}

impl With<Position> for PosColor {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::FORMAT,
    };
}

impl With<Color> for PosColor {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Color::FORMAT,
    };
}

/// Vertex format with position, RGBA8 color and normal attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PosColorNorm {
    /// Position of the vertex in 3D space.
    pub position: Vector3<f32>,
    /// RGBA color value of the vertex.
    pub color: [f32; 4],
    /// Normal vector of the vertex.
    pub normal: Vector3<f32>,
}

unsafe impl Pod for PosColorNorm {}

impl VertexFormat for PosColorNorm {
    const ATTRIBUTES: Attributes<'static> = &[
        (Position::NAME, <Self as With<Position>>::FORMAT),
        (Color::NAME, <Self as With<Color>>::FORMAT),
        (Normal::NAME, <Self as With<Normal>>::FORMAT),
    ];
}

impl With<Position> for PosColorNorm {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::FORMAT,
    };
}

impl With<Color> for PosColorNorm {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Color::FORMAT,
    };
}

impl With<Normal> for PosColorNorm {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Color::SIZE,
        format: Normal::FORMAT,
    };
}

/// Vertex format with position and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PosTex {
    /// Position of the vertex in 3D space.
    pub position: Vector3<f32>,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: Vector2<f32>,
}

unsafe impl Pod for PosTex {}

impl VertexFormat for PosTex {
    const ATTRIBUTES: Attributes<'static> = &[
        (Position::NAME, <Self as With<Position>>::FORMAT),
        (TexCoord::NAME, <Self as With<TexCoord>>::FORMAT),
    ];
}

impl With<Position> for PosTex {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::FORMAT,
    };
}

impl With<TexCoord> for PosTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: TexCoord::FORMAT,
    };
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PosNormTex {
    /// Position of the vertex in 3D space.
    pub position: Vector3<f32>,
    /// Normal vector of the vertex.
    pub normal: Vector3<f32>,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: Vector2<f32>,
}

unsafe impl Pod for PosNormTex {}

impl VertexFormat for PosNormTex {
    const ATTRIBUTES: Attributes<'static> = &[
        (Position::NAME, <Self as With<Position>>::FORMAT),
        (Normal::NAME, <Self as With<Normal>>::FORMAT),
        (TexCoord::NAME, <Self as With<TexCoord>>::FORMAT),
    ];
}

impl With<Position> for PosNormTex {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::FORMAT,
    };
}

impl With<Normal> for PosNormTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Normal::FORMAT,
    };
}

impl With<TexCoord> for PosNormTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Normal::SIZE,
        format: TexCoord::FORMAT,
    };
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PosNormTangTex {
    /// Position of the vertex in 3D space.
    pub position: Vector3<f32>,
    /// Normal vector of the vertex.
    pub normal: Vector3<f32>,
    /// Tangent vector of the vertex.
    pub tangent: Vector3<f32>,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: Vector2<f32>,
}

unsafe impl Pod for PosNormTangTex {}

impl VertexFormat for PosNormTangTex {
    const ATTRIBUTES: Attributes<'static> = &[
        (Position::NAME, <Self as With<Position>>::FORMAT),
        (Normal::NAME, <Self as With<Normal>>::FORMAT),
        (Tangent::NAME, <Self as With<Tangent>>::FORMAT),
        (TexCoord::NAME, <Self as With<TexCoord>>::FORMAT),
    ];
}

impl With<Position> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::FORMAT,
    };
}

impl With<Normal> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Normal::FORMAT,
    };
}

impl With<Tangent> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Normal::SIZE,
        format: Tangent::FORMAT,
    };
}

impl With<TexCoord> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Normal::SIZE + Tangent::SIZE,
        format: TexCoord::FORMAT,
    };
}

/// Vertex combo
pub type VertexBufferCombination = (
    Vec<Separate<Position>>,
    Option<Vec<Separate<Color>>>,
    Option<Vec<Separate<TexCoord>>>,
    Option<Vec<Separate<Normal>>>,
    Option<Vec<Separate<Tangent>>>,
);

/// Allows to query specific `Attribute`s of `VertexFormat`
pub trait Query<T>: VertexFormat {
    /// Attributes from tuple `T`
    const QUERIED_ATTRIBUTES: Attributes<'static>;
}

macro_rules! impl_query {
    ($($a:ident),*) => {
        impl<VF $(,$a)*> Query<($($a,)*)> for VF
            where VF: VertexFormat,
            $(
                $a: Attribute,
                VF: With<$a>,
            )*
        {
            const QUERIED_ATTRIBUTES: Attributes<'static> = &[
                $(
                    ($a::NAME, <VF as With<$a>>::FORMAT),
                )*
            ];
        }

        impl_query!(@ $($a),*);
    };
    (@) => {};
    (@ $head:ident $(,$tail:ident)*) => {
        impl_query!($($tail),*);
    };
}

impl_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
