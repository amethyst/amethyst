
use gfx;
use mopa;

pub trait Framebuffer: mopa::Any {}
mopafy!(Framebuffer);

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

/// A simple output containing both a Color and a Depth target
pub struct ColorBuffer<R: gfx::Resources> {
    pub color: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub output_depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

impl<R: gfx::Resources> Framebuffer for ColorBuffer<R> {}

/// A geometry buffer is used in a deferred pipeline
pub struct GeometryBuffer<R: gfx::Resources> {
    pub normal: gfx::handle::RenderTargetView<R, [f32; 4]>,
    pub ka: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub kd: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub depth: gfx::handle::DepthStencilView<R, DepthFormat>,

    pub texture_normal: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub texture_ka: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub texture_kd: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub texture_depth: gfx::handle::ShaderResourceView<R, f32>,
}

impl<R: gfx::Resources> GeometryBuffer<R> {
    /// Create a new GeometryBuffer with the supplied factory
    /// the buffer will be allocated to the supplied width and height
    pub fn new<F>(factory: &mut F, (width, height): (u16, u16)) -> Self
        where F: gfx::Factory<R>
    {
        let (_, texture_normal,  normal) = factory.create_render_target(width, height).unwrap();
        let (_, texture_ka,  ka) = factory.create_render_target(width, height).unwrap();
        let (_, texture_kd,  kd) = factory.create_render_target(width, height).unwrap();
        let (_, texture_depth, depth) = factory.create_depth_stencil(width, height).unwrap();

        GeometryBuffer{
            normal: normal,
            kd: kd,
            ka: ka,
            depth: depth,
            texture_normal: texture_normal,
            texture_ka: texture_ka,
            texture_kd: texture_kd,
            texture_depth: texture_depth
        }
    }
}

impl<R: gfx::Resources> Framebuffer for GeometryBuffer<R> {}