//! Built-in vertex formats.
use std::fmt::Debug;

use gfx_hal::format::{BufferFormat, ChannelType, Format, Formatted, SurfaceType, Vec2, Vec3, Vec4};
use gfx_hal::pso::Element;
use gfx_hal::memory::Pod;

/// Format for vertex attribute
pub type AttributeFormat = Element<Format>;

/// Slice of attributes
pub type Attributes<'a> = &'a [(&'a str, AttributeFormat)];
pub type AttributesSet<'a> = &'a [Attributes<'a>];

/// Trait for vertex attributes to implement
pub trait Attribute: BufferFormat + Debug + PartialEq + Pod + Send + Sync {
    /// Name of the attribute
    /// It is used to bind to the attributes in shaders
    const NAME: &'static str;

    /// Size of the attribue
    /// TODO: Remove when `std::mem_size_of` became const fn
    const SIZE: u32; // Has to be equal to `std::mem::size_of::<Self>() as u32`
}

/// Type for position attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(Vec3<f32>);
impl Formatted for Position {
    type Surface = <Vec3<f32> as Formatted>::Surface;
    type Channel = <Vec3<f32> as Formatted>::Channel;
    type View = <Vec3<f32> as Formatted>::View;
    const SELF: Format = Vec3::<f32>::SELF;
}
unsafe impl Pod for Position {}
impl Attribute for Position {
    const NAME: &'static str = "position";
    const SIZE: u32 = 12;
}

/// Type for color attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(Vec4<f32>);
impl Formatted for Color {
    type Surface = <Vec4<f32> as Formatted>::Surface;
    type Channel = <Vec4<f32> as Formatted>::Channel;
    type View = <Vec4<f32> as Formatted>::View;
    const SELF: Format = Vec4::<f32>::SELF;
}
unsafe impl Pod for Color {}
impl Attribute for Color {
    const NAME: &'static str = "color";
    const SIZE: u32 = 16;
}

/// Type for texture coord attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexCoord(Vec2<f32>);
impl Formatted for TexCoord {
    type Surface = <Vec2<f32> as Formatted>::Surface;
    type Channel = <Vec2<f32> as Formatted>::Channel;
    type View = <Vec2<f32> as Formatted>::View;
    const SELF: Format = Vec2::<f32>::SELF;
}
unsafe impl Pod for TexCoord {}
impl Attribute for TexCoord {
    const NAME: &'static str = "tex_coord";
    const SIZE: u32 = 8;
}

/// Type for texture coord attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal(Vec3<f32>);
impl Formatted for Normal {
    type Surface = <Vec3<f32> as Formatted>::Surface;
    type Channel = <Vec3<f32> as Formatted>::Channel;
    type View = <Vec3<f32> as Formatted>::View;
    const SELF: Format = Vec3::<f32>::SELF;
}
unsafe impl Pod for Normal {}
impl Attribute for Normal {
    const NAME: &'static str = "normal";
    const SIZE: u32 = 12;
}

/// Type for tangent attribute of vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tangent(Vec3<f32>);
impl Formatted for Tangent {
    type Surface = <Vec3<f32> as Formatted>::Surface;
    type Channel = <Vec3<f32> as Formatted>::Channel;
    type View = <Vec3<f32> as Formatted>::View;
    const SELF: Format = Vec3::<f32>::SELF;
}
unsafe impl Pod for Tangent {}
impl Attribute for Tangent {
    const NAME: &'static str = "tangent";
    const SIZE: u32 = 12;
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

impl<T> VertexFormat for T
where
    T: Attribute,
{
    const ATTRIBUTES: Attributes<'static> = &[
        (
            T::NAME,
            Element {
                format: T::SELF,
                offset: 0,
            },
        ),
    ];
}

/// Trait implemented by all valid vertex formats for each field
pub trait With<F: Attribute>: VertexFormat {
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

impl VertexFormat for PosColor {
    const ATTRIBUTES: Attributes<'static> = &[
        (Position::NAME, <Self as With<Position>>::FORMAT),
        (Color::NAME, <Self as With<Color>>::FORMAT),
    ];
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
    pub position: Vec3<f32>,
    /// UV texture coordinates used by the vertex.
    pub tex_coord: Vec2<f32>,
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

/// Vertex format with position, normal, and UV texture coordinate attributes.
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
