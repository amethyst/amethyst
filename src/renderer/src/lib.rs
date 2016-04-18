#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]
#![allow(dead_code)]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
extern crate glutin;
extern crate cgmath;

mod forward;
mod gbuffer;
mod wireframe;

use std::collections::HashMap;

use gfx::Slice;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use cgmath::{Matrix4, SquareMatrix};

use gbuffer::{GBufferTarget, GBufferShaderResource};

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
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

    wireframe_pipeline: wireframe::WireframePipeline<R>,
    command_buffer: gfx::Encoder<R, C>
}

// placeholder
gfx_vertex_struct!( VertexPosNormal {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Normal",
});

impl<R, C> Renderer<R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    /// Create a new Render pipline
    pub fn new<F>(factory: &mut F, combuf: C) -> Renderer<R, C>
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

            command_buffer: combuf.into()
        }
    }

    /// submit an operation to be rendered
    fn submit_op(&mut self,
                 scenes: &HashMap<String, Scene<R, VertexPosNormal>>,
                 screen: &ScreenOutput<R>,
                 op: &Operation)
    {
        match op {
            &Operation::Clear(colour, depth) => {
                if depth {
                    self.command_buffer.clear_depth(&screen.output_depth, 1.0);
                }
                self.command_buffer.clear(&screen.output, colour);
            }
            &Operation::Wireframe(ref camera, ref fragments) => {
                let scene = &scenes[fragments];

                // every entity gets drawn
                for e in &scene.fragments {
                    self.command_buffer.draw(
                        &e.slice,
                        &self.wireframe_pipeline,
                        &wireframe::wireframe::Data{
                            vbuf: e.buffer.clone(),
                            ka: e.ka,
                            model: e.transform,
                            view: camera.view,
                            proj: camera.projection,
                            out_ka: screen.output.clone()
                        }
                    );
                }
            }
            &Operation::FlatShading(ref camera, ref fragments) => {
                let scene = &scenes[fragments];

                // every entity gets drawn
                for e in &scene.fragments {
                    self.command_buffer.draw(
                        &e.slice,
                        &self.pipeline_foward,
                        &forward::flat::Data{
                            vbuf: e.buffer.clone(),
                            ka: e.ka,
                            model: e.transform,
                            view: camera.view,
                            proj: camera.projection,
                            out_ka: screen.output.clone(),
                            out_depth: screen.output_depth.clone()
                        }
                    );
                }
            }
        }
    }

    /// Raster the pass
    fn submit_pass(&mut self, scenes: &HashMap<String, Scene<R, VertexPosNormal>>, pass: &Pass<R>) {
        for op in &pass.ops {
            self.submit_op(&scenes, &pass.output, op);
        }
    }

    /// Raster the frame
    pub fn submit<D>(&mut self, frame: &Frame<R, VertexPosNormal>, device: &mut D)
        where D: gfx::Device<Resources=R, CommandBuffer=C>
    {
        for pass in &frame.passes {
            self.submit_pass(&frame.scenes, &pass);
        }
        self.command_buffer.flush(device);
        device.cleanup();
    }
}

// placeholder Entity
pub struct Fragment<R: gfx::Resources, T> {
    pub transform: [[f32; 4]; 4],
    pub buffer: gfx::handle::Buffer<R, T>,
    pub slice: gfx::Slice<R>,
    /// ambient colour
    pub ka: [f32; 4],
    /// diffuse colour
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

/// Render target
pub struct ScreenOutput<R: gfx::Resources> {
    pub output: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub output_depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

pub struct Scene<R: gfx::Resources, T> {
    pub fragments: Vec<Fragment<R, T>>,
    pub lights: Vec<Light>
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}

pub enum Operation {
    Clear([f32; 4], bool),
    Wireframe(Camera, String),
    FlatShading(Camera, String),
}

pub struct Pass<R: gfx::Resources> {
    pub output: ScreenOutput<R>,
    pub ops: Vec<Operation>
}

/// The render job submission
pub struct Frame<R: gfx::Resources, T> {
    pub passes: Vec<Pass<R>>,
    pub scenes: HashMap<String, Scene<R, T>>
}

