//! Built-in vertex formats.

use std::fmt::Debug;

use gfx_hal::format::{AsFormat, Format};
use gfx_hal::memory::Pod;
use gfx_hal::pso::{ElemStride, Element};

/// Format for vertex attribute
pub type AttributeFormat = Element<Format>;

/// Slice of attributes
pub type Attributes<'a> = &'a [(u32, &'a str, AttributeFormat)];

/// Trait for vertex attributes to implement
pub trait Attribute: AsFormat + Debug + PartialEq + Pod + Send + Sync {
    /// Binding index of attribute type
    /// It is used to bind to the attributes in shaders
    const BINDING: u32;

    /// Name of the attribute
    const NAME: &'static str;

    /// Size of the attribute
    /// TODO: Remove when `std::mem_size_of` became const fn
    const SIZE: ElemStride; // Has to be equal to `std::mem::size_of::<Self>() as ElemStride`
}

/// Type for position attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub [f32; 3]);
impl<T> From<T> for Position
where
    T: Into<[f32; 3]>,
{
    fn from(from: T) -> Self {
        Position(from.into())
    }
}

impl AsFormat for Position {
    const SELF: Format = <[f32; 3] as AsFormat>::SELF;
}
unsafe impl Pod for Position {}
impl Attribute for Position {
    const BINDING: u32 = 0;
    const NAME: &'static str = "position";
    const SIZE: ElemStride = 12;
}

/// Type for color attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub [f32; 4]);
impl<T> From<T> for Color
where
    T: Into<[f32; 4]>,
{
    fn from(from: T) -> Self {
        Color(from.into())
    }
}

impl AsFormat for Color {
    const SELF: Format = <[f32; 4] as AsFormat>::SELF;
}
unsafe impl Pod for Color {}
impl Attribute for Color {
    const BINDING: u32 = 1;
    const NAME: &'static str = "color";
    const SIZE: ElemStride = 16;
}

/// Type for texture coord attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal(pub [f32; 3]);
impl<T> From<T> for Normal
where
    T: Into<[f32; 3]>,
{
    fn from(from: T) -> Self {
        Normal(from.into())
    }
}
impl AsFormat for Normal {
    const SELF: Format = <[f32; 3] as AsFormat>::SELF;
}
unsafe impl Pod for Normal {}
impl Attribute for Normal {
    const BINDING: u32 = 2;
    const NAME: &'static str = "normal";
    const SIZE: ElemStride = 12;
}

/// Type for tangent attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tangent(pub [f32; 3]);
impl<T> From<T> for Tangent
where
    T: Into<[f32; 3]>,
{
    fn from(from: T) -> Self {
        Tangent(from.into())
    }
}
impl AsFormat for Tangent {
    const SELF: Format = <[f32; 3] as AsFormat>::SELF;
}
unsafe impl Pod for Tangent {}
impl Attribute for Tangent {
    const BINDING: u32 = 3;
    const NAME: &'static str = "tangent";
    const SIZE: ElemStride = 12;
}

/// Type for texture coord attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexCoord(pub [f32; 2]);
impl<T> From<T> for TexCoord
where
    T: Into<[f32; 2]>,
{
    fn from(from: T) -> Self {
        TexCoord(from.into())
    }
}
impl AsFormat for TexCoord {
    const SELF: Format = <[f32; 2] as AsFormat>::SELF;
}
unsafe impl Pod for TexCoord {}
impl Attribute for TexCoord {
    const BINDING: u32 = 4;
    const NAME: &'static str = "tex_coord";
    const SIZE: ElemStride = 8;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VertexFormat<'a> {
    pub attributes: Attributes<'a>,
    pub stride: ElemStride,
}

pub type VertexFormatSet<'a> = &'a [VertexFormat<'a>];

/// Trait implemented by all valid vertex formats.
pub trait VertexFormatted: Pod + Sized + Send + Sync {
    /// List of all attributes formats with name and offset.
    const VERTEX_FORMAT: VertexFormat<'static>;

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

impl<T> VertexFormatted for T
where
    T: Attribute,
{
    const VERTEX_FORMAT: VertexFormat<'static> = VertexFormat {
        attributes: &[
            (
                T::BINDING,
                T::NAME,
                Element {
                    format: T::SELF,
                    offset: 0,
                },
            ),
        ],
        stride: T::SIZE,
    };
}

/// Trait implemented by all valid vertex formats for each field
pub trait With<F: Attribute>: VertexFormatted {
    /// Individual format of the attribute for this vertex format
    const FORMAT: AttributeFormat;
}

impl<T> With<T> for T
where
    T: Attribute,
{
    const FORMAT: AttributeFormat = Element {
        format: T::SELF,
        offset: 0,
    };
}

/// Vertex format with position and RGBA8 color attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosColor {
    /// Position of the vertex in 3D space.
    pub position: Position,
    /// RGBA color value of the vertex.
    pub color: Color,
}

unsafe impl Pod for PosColor {}

impl VertexFormatted for PosColor {
    const VERTEX_FORMAT: VertexFormat<'static> = VertexFormat {
        attributes: &[
            (
                Position::BINDING,
                Position::NAME,
                <Self as With<Position>>::FORMAT,
            ),
            (Color::BINDING, Color::NAME, <Self as With<Color>>::FORMAT),
        ],
        stride: Position::SIZE + Color::SIZE,
    };
}

impl With<Position> for PosColor {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::SELF,
    };
}

impl With<Color> for PosColor {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Color::SELF,
    };
}

/// Vertex format with position and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosTex {
    /// Position of the vertex in 3D space.
    pub position: [f32; 3],
    /// UV texture coordinates used by the vertex.
    pub tex_coord: [f32; 2],
}

unsafe impl Pod for PosTex {}

impl VertexFormatted for PosTex {
    const VERTEX_FORMAT: VertexFormat<'static> = VertexFormat {
        attributes: &[
            (
                Position::BINDING,
                Position::NAME,
                <Self as With<Position>>::FORMAT,
            ),
            (
                TexCoord::BINDING,
                TexCoord::NAME,
                <Self as With<TexCoord>>::FORMAT,
            ),
        ],
        stride: Position::SIZE + TexCoord::SIZE,
    };
}

impl With<Position> for PosTex {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::SELF,
    };
}

impl With<TexCoord> for PosTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: TexCoord::SELF,
    };
}

/// Vertex format with position, normal, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosNormTex {
    /// Position of the vertex in 3D space.
    pub position: Position,
    /// Normal vector of the vertex.
    pub normal: Normal,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: TexCoord,
}

unsafe impl Pod for PosNormTex {}

impl VertexFormatted for PosNormTex {
    const VERTEX_FORMAT: VertexFormat<'static> = VertexFormat {
        attributes: &[
            (
                Position::BINDING,
                Position::NAME,
                <Self as With<Position>>::FORMAT,
            ),
            (
                Normal::BINDING,
                Normal::NAME,
                <Self as With<Normal>>::FORMAT,
            ),
            (
                TexCoord::BINDING,
                TexCoord::NAME,
                <Self as With<TexCoord>>::FORMAT,
            ),
        ],
        stride: Position::SIZE + Normal::SIZE + TexCoord::SIZE,
    };
}

impl With<Position> for PosNormTex {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::SELF,
    };
}

impl With<Normal> for PosNormTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Normal::SELF,
    };
}

impl With<TexCoord> for PosNormTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Normal::SIZE,
        format: TexCoord::SELF,
    };
}

/// Vertex format with position, normal, tangent, and UV texture coordinate attributes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosNormTangTex {
    /// Position of the vertex in 3D space.
    pub position: Position,
    /// Normal vector of the vertex.
    pub normal: Normal,
    /// Tangent vector of the vertex.
    pub tangent: Tangent,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: TexCoord,
}

unsafe impl Pod for PosNormTangTex {}

impl VertexFormatted for PosNormTangTex {
    const VERTEX_FORMAT: VertexFormat<'static> = VertexFormat {
        attributes: &[
            (
                Position::BINDING,
                Position::NAME,
                <Self as With<Position>>::FORMAT,
            ),
            (
                Normal::BINDING,
                Normal::NAME,
                <Self as With<Normal>>::FORMAT,
            ),
            (
                Tangent::BINDING,
                Tangent::NAME,
                <Self as With<Tangent>>::FORMAT,
            ),
            (
                TexCoord::BINDING,
                TexCoord::NAME,
                <Self as With<TexCoord>>::FORMAT,
            ),
        ],
        stride: Position::SIZE + Normal::SIZE + Tangent::SIZE + TexCoord::SIZE,
    };
}

impl With<Position> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: 0,
        format: Position::SELF,
    };
}

impl With<Normal> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE,
        format: Normal::SELF,
    };
}

impl With<Tangent> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Normal::SIZE,
        format: Tangent::SELF,
    };
}

impl With<TexCoord> for PosNormTangTex {
    const FORMAT: AttributeFormat = Element {
        offset: Position::SIZE + Normal::SIZE + Tangent::SIZE,
        format: TexCoord::SELF,
    };
}

/// Allows to query specific `Attribute`s of `VertexFormatted`
pub trait Query<T>: VertexFormatted {
    /// Attributes from tuple `T`
    const QUERIED_ATTRIBUTES: Attributes<'static>;
}

macro_rules! impl_query {
    ($($a:ident),*) => {
        impl<VF $(,$a)*> Query<($($a,)*)> for VF
            where VF: VertexFormatted,
            $(
                $a: Attribute,
                VF: With<$a>,
            )*
        {
            const QUERIED_ATTRIBUTES: Attributes<'static> = &[
                $(
                    ($a::BINDING, $a::NAME, <VF as With<$a>>::FORMAT),
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

impl_query!(
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z
);
