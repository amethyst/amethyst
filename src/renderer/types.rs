//! Special types for representing cross-API GPU data.

pub struct BufferInfo {
    /// Size in bytes.
    pub size: usize,
}

#[derive(Clone)]
pub enum Buffer {
    Index(u64),
    Uniform(u64),
    Vertex(u64),
}

#[derive(Clone)]
pub enum Shader {
    Compute(u64),
    Fragment(u64),
    Geometry(u64),
    Vertex(u64),
}

pub enum Blend {
    One,
    OneMinusConstAlpha,
    OneMinusConstColor,
    OneMinusDestAlpha,
    OneMinusDestColor,
    OneMinusSourceAlpha,
    OneMinusSourceColor,
    ConstantAlpha,
    ConstantColor,
    DestAlpha,
    DestColor,
    SourceAlpha,
    SourceAlphaSaturate,
    SourceColor,
    Zero,
}

pub enum BlendFunc {
    Add,
    Max,
    Min,
    ReverseSub,
    Sub,
}

#[derive(Clone)]
pub enum ClearMask {
    Color,
    Depth,
    Stencil,
}

pub enum CompareFunc {
    Always,
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Never,
    NotEqual,
}

pub enum CullMode {
    None,
    Back,
    Front,
}

pub struct DepthStencilOp {
    pub depth_fail: StencilOp,
    pub fail: StencilOp,
    pub pass: StencilOp,
    pub stencil_func: CompareFunc,
    pub reference_value: u8
}

pub enum FillMode {
    Solid,
    Wireframe,
}

pub enum LogicOp {
    And,
    AndReverse,
    AndInverted,
    Copy,
    CopyInverted,
    Clear,
    Equiv,
    Invert,
    Nand,
    NoOp,
    Nor,
    Or,
    OrInverted,
    OrReverse,
    Set,
    Xor,
}

pub enum Primitive {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
    Quads,
}

pub struct ScissorBox {
    pub origin: [f32; 2],
    pub size: [f32; 2]
}

pub enum StencilOp {
    DecrementClamp,
    DecrementWrap,
    IncrementClamp,
    IncrementWrap,
    Invert,
    Keep,
    Replace,
    Zero,
}

pub struct ShaderSet {
    pub fragment: Shader,
    pub geometry: Shader,
    pub vertex: Shader,
}

pub enum Target {
    DepthStencil(u64),
    Render(u64),
}

pub struct Viewport {
    pub origin: [f32; 2],
    pub size: [f32; 2],
    pub min_depth: f32,
    pub max_depth: f32,
}

pub enum Winding {
    CW,
    CCW,
}

