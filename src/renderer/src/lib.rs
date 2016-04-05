#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
extern crate glutin;
extern crate cgmath;

mod forward;
mod gbuffer;
mod wireframe;

use gfx::Slice;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use cgmath::{Matrix4, SquareMatrix};

pub use forward::VertexPosNormal;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub struct Renderer<R: gfx::Resources> {
    pipeline_foward: forward::FlatPipeline<R>,
    flat_uniform_vs: gfx::handle::Buffer<R, forward::VertexUniforms>,
    flat_uniform_fs: gfx::handle::Buffer<R, forward::FlatFragmentUniforms>,

    gbuf_target: GBufferTarget<R>,
    gbuf_texture: GBufferShaderResource<R>,

    blit_mesh: Buffer<R, gbuffer::Vertex>,
    blit_slice: Slice<R>,
    blit_pipeline: gbuffer::BlitPipeline<R>,
    blit_sampler: gfx::handle::Sampler<R>,

    light_pipeline: gbuffer::LightPipeline<R>,

    wireframe_pipeline: wireframe::WireframePipeline<R>
}

struct GBufferTarget<R: gfx::Resources> {
    normal: gfx::handle::RenderTargetView<R, [f32; 4]>,
    ka: gfx::handle::RenderTargetView<R, ColorFormat>,
    kd: gfx::handle::RenderTargetView<R, ColorFormat>,
    depth: gfx::handle::DepthStencilView<R, gfx::format::DepthStencil>,
}

struct GBufferShaderResource<R: gfx::Resources> {
    normal: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    ka: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    kd: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    depth: gfx::handle::ShaderResourceView<R, f32>,
}

impl<R> GBufferTarget<R>
    where R: gfx::Resources
{
    fn new<F>(factory: &mut F, (width, height): (u16, u16)) -> (Self, GBufferShaderResource<R>)
        where F: gfx::Factory<R>
    {
        let (_, texture_normal,  normal) = factory.create_render_target(width, height).unwrap();
        let (_, texture_ka,  ka) = factory.create_render_target(width, height).unwrap();
        let (_, texture_kd,  kd) = factory.create_render_target(width, height).unwrap();
        let (_, texture_depth, depth) = factory.create_depth_stencil(width, height).unwrap();

        (
            GBufferTarget{
                normal: normal,
                kd: kd,
                ka: ka,
                depth: depth
            },
            GBufferShaderResource{
                normal: texture_normal,
                ka: texture_ka,
                kd: texture_kd,
                depth: texture_depth
            }
        )
    }
}

impl<R> Renderer<R>
    where R: gfx::Resources
{
    pub fn new<F>(factory: &mut F) -> Renderer<R>
        where F: gfx::Factory<R>
    {
        let pipeline_foward = forward::create_flat_pipeline(factory);
        let (gbuf_target, gbuf_texture) = GBufferTarget::new(factory, (800, 600));
        let flat_uniform_vs = factory.create_constant_buffer(1);
        let flat_uniform_fs = factory.create_constant_buffer(1);

        let (buffer, slice) = gbuffer::create_mesh(factory);
        let blit_pipeline = gbuffer::create_blit_pipeline(factory);

        let blit_sampler = factory.create_sampler(
            gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Scale,
                                       gfx::tex::WrapMode::Clamp)
        );

        Renderer {
            pipeline_foward: pipeline_foward,
            gbuf_target: gbuf_target,
            gbuf_texture: gbuf_texture,
            flat_uniform_vs: flat_uniform_vs,
            flat_uniform_fs: flat_uniform_fs,

            blit_mesh: buffer,
            blit_slice: slice,
            blit_pipeline: blit_pipeline,
            blit_sampler: blit_sampler,

            light_pipeline: gbuffer::create_light_pipline(factory),
            wireframe_pipeline: wireframe::create_wireframe_pipeline(factory),
        }
    }

    pub fn wireframe<C>(&mut self,
                     scene: &Scene<R, VertexPosNormal>,
                     encoder: &mut gfx::Encoder<R, C>,
                     output: &gfx::handle::RenderTargetView<R, ColorFormat>)
        where C: gfx::CommandBuffer<R>
    {
        // clear the gbuffer
        encoder.clear(&output, [0.; 4]);


        // every entity gets drawn
        for e in &scene.entities {
            encoder.draw(
                &e.slice,
                &self.wireframe_pipeline,
                &wireframe::wireframe::Data{
                    vbuf: e.buffer.clone(),
                    ka: e.ka,
                    model: e.transform,
                    view: scene.view,
                    proj: scene.projection,
                    out_ka: output.clone()
                }
            );
        }
    }


    pub fn render<C>(&mut self,
                     scene: &Scene<R, VertexPosNormal>,
                     encoder: &mut gfx::Encoder<R, C>,
                     output: &gfx::handle::RenderTargetView<R, ColorFormat>,
                     output_depth: &gfx::handle::DepthStencilView<R, DepthFormat>)
        where C: gfx::CommandBuffer<R>
    {

        // clear the gbuffer
        encoder.clear(&self.gbuf_target.normal, [0.; 4]);
        encoder.clear(&self.gbuf_target.ka, [0.; 4]);
        encoder.clear(&self.gbuf_target.kd, [0.; 4]);
        encoder.clear_depth(&self.gbuf_target.depth, 1.0);
        encoder.clear_stencil(&output_depth, 0);

        // every entity gets drawn
        for e in &scene.entities {
            encoder.update_constant_buffer(&self.flat_uniform_vs,
                &forward::VertexUniforms {
                    view: scene.view,
                    proj: scene.projection,
                    model: e.transform
                }
            );
            encoder.update_constant_buffer(&self.flat_uniform_fs,
                &forward::FlatFragmentUniforms{
                    ka: e.ka,
                    kd: e.kd
                }
            );

            encoder.draw(
                &e.slice,
                &self.pipeline_foward,
                &forward::flat::Data {
                    vbuf: e.buffer.clone(),
                    uniform_vs: self.flat_uniform_vs.clone(),
                    uniform_fs: self.flat_uniform_fs.clone(),
                    out_normal: self.gbuf_target.normal.clone(),
                    out_ka: self.gbuf_target.ka.clone(),
                    out_kd: self.gbuf_target.kd.clone(),
                    out_depth: self.gbuf_target.depth.clone()
                }
            );
        }

        // blit the gbuffer to the screen
        encoder.draw(
            &self.blit_slice,
            &self.blit_pipeline,
            &gbuffer::blit::Data {
                vbuf: self.blit_mesh.clone(),
                source: (self.gbuf_texture.ka.clone(), self.blit_sampler.clone()),
                out: output.clone(),
            }
        );

        for l in &scene.lights {
            encoder.draw(
                &self.blit_slice,
                &self.light_pipeline,
                &gbuffer::light::Data {
                    vbuf: self.blit_mesh.clone(),
                    kd: (self.gbuf_texture.kd.clone(), self.blit_sampler.clone()),
                    normal: (self.gbuf_texture.normal.clone(), self.blit_sampler.clone()),
                    depth: (self.gbuf_texture.depth.clone(), self.blit_sampler.clone()),
                    out: output.clone(),
                    color: l.color,
                    center: [l.center[0], l.center[1], l.center[2], 1.],
                    propagation: [
                        l.propagation_constant,
                        l.propagation_linear,
                        l.propagation_r_square,
                    ],
                    inv_proj: Matrix4::from(scene.projection).invert().unwrap().into(),
                    inv_view: Matrix4::from(scene.view).invert().unwrap().into(),
                    proj: scene.projection,
                    viewport: [0., 0., 800., 600.]
                }
            );
        }
    }
}

// placeholder Entity
pub struct Entity<R: gfx::Resources, T> {
    pub transform: [[f32; 4]; 4],
    pub buffer: gfx::handle::Buffer<R, T>,
    pub slice: gfx::Slice<R>,
    // ambient colour
    pub ka: [f32; 4],
    // diffuse colour
    pub kd: [f32; 4]
}

// placeholder light
pub struct Light {
    // clip scale
    pub center: [f32; 3],
    pub radius: f32,

    pub color: [f32; 4],
    // color * (pc + pl / r + pc / (r^2))
    pub propagation_constant: f32,
    pub propagation_linear: f32,
    pub propagation_r_square: f32,

}

// this is a placeholder until we get a working ECS
pub struct Scene<R: gfx::Resources, T> {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],

    pub entities: Vec<Entity<R, T>>,
    pub lights: Vec<Light>
}

