use gfx;
use mopa;

/// A Target or a RenderTarget is any object that
/// can be the target of a Layer This is normally
/// a framebuffer
pub trait Target: mopa::Any + Sync {}
mopafy!(Target);

/// Placeholder Color format
pub type ColorFormat = gfx::format::Rgba8;
/// Placeholder Depth Format
pub type DepthFormat = gfx::format::DepthStencil;

/// A simple output containing both a Color and a Depth target
pub struct ColorBuffer<R: gfx::Resources> {
    /// The color render target
    pub color: gfx::handle::RenderTargetView<R, ColorFormat>,
    /// The depth buffer
    pub output_depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

impl<R: gfx::Resources> Target for ColorBuffer<R> {}

/// A geometry buffer that is used in a deferred pipeline.
/// TODO: Why both `ka` and `texture_ka`, etc?
pub struct GeometryBuffer<R: gfx::Resources> {
    /// Contains the Normals as a f32x4
    pub normal: gfx::handle::RenderTargetView<R, [f32; 4]>,

    /// Contains the ambient color
    pub ka: gfx::handle::RenderTargetView<R, ColorFormat>,

    /// Contains the diffuse color
    pub kd: gfx::handle::RenderTargetView<R, ColorFormat>,

    /// Contains the specular color
    pub ks: gfx::handle::RenderTargetView<R, ColorFormat>,

    /// Contains the depth buffer
    pub depth: gfx::handle::DepthStencilView<R, DepthFormat>,

    /// The normal buffer as a texture
    pub texture_normal: gfx::handle::ShaderResourceView<R, [f32; 4]>,

    /// The ambient color as texture
    pub texture_ka: gfx::handle::ShaderResourceView<R, [f32; 4]>,

    /// The diffuse color as a texture
    pub texture_kd: gfx::handle::ShaderResourceView<R, [f32; 4]>,

    /// The specular color as a texture
    pub texture_ks: gfx::handle::ShaderResourceView<R, [f32; 4]>,

    /// The depth buffer as a texture
    pub texture_depth: gfx::handle::ShaderResourceView<R, f32>,
}

impl<R: gfx::Resources> GeometryBuffer<R> {
    /// Create a new GeometryBuffer with the supplied factory
    /// the buffer will be allocated to the supplied width and height
    pub fn new<F>(factory: &mut F, (width, height): (u16, u16)) -> Self
        where F: gfx::Factory<R>
    {
        let (_, texture_normal, normal) = factory.create_render_target(width, height).unwrap();
        let (_, texture_ka, ka) = factory.create_render_target(width, height).unwrap();
        let (_, texture_kd, kd) = factory.create_render_target(width, height).unwrap();
        let (_, texture_ks, ks) = factory.create_render_target(width, height).unwrap();
        let (_, texture_depth, depth) = factory.create_depth_stencil(width, height).unwrap();

        GeometryBuffer {
            normal: normal,
            ka: ka,
            kd: kd,
            ks: ks,
            depth: depth,
            texture_normal: texture_normal,
            texture_ka: texture_ka,
            texture_kd: texture_kd,
            texture_ks: texture_ks,
            texture_depth: texture_depth,
        }
    }
}

impl<R: gfx::Resources> Target for GeometryBuffer<R> {}
