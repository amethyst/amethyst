#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]
#![allow(dead_code)]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
#[macro_use]
extern crate mopa;

extern crate glutin;
extern crate cgmath;

pub mod forward;
mod gbuffer;
mod wireframe;

use std::any::TypeId;
use std::collections::HashMap;

use mopa::Any;

pub use gbuffer::{GBuffer, Draw, BlitAmbiant, Lighting};

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    command_buffer: gfx::Encoder<R, C>,
    methods: HashMap<(TypeId, TypeId), Box<Fn(&Box<Operation>, &Target, &Frame<R>, &mut gfx::Encoder<R, C>)>>
}

// placeholder
gfx_vertex_struct!( VertexPosNormal {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Normal",
});

impl<R, C> Renderer<R, C>
    where R: gfx::Resources,
          <R as gfx::Resources>::RenderTargetView: Any,
          <R as gfx::Resources>::Texture: Any,
          <R as gfx::Resources>::DepthStencilView: Any,
          <R as gfx::Resources>::ShaderResourceView: Any,
          <R as gfx::Resources>::Buffer: Any,
          R: 'static,
          C: gfx::CommandBuffer<R>
{
    /// Create a new Render pipline
    pub fn new(combuf: C) -> Renderer<R, C> {
        Renderer {
            command_buffer: combuf.into(),
            methods: HashMap::new()
        }
    }

    /// Load all known methods
    pub fn load_all<F>(&mut self, factory: &mut F)
        where F: gfx::Factory<R>
    {
        self.add_method(forward::Clear);
        self.add_method(forward::FlatShading::new(factory));
        self.add_method(forward::Wireframe::new(factory));

        self.add_method(gbuffer::Clear);
        self.add_method(gbuffer::DrawMethod::new(factory));
        self.add_method(gbuffer::BlitAmbiantMethod::new(factory));
        self.add_method(gbuffer::LightingMethod::new(factory));
    }

    /// Add a method to the table of available methods
    pub fn add_method<A, T, P>(&mut self, p: P)
        where P: Method<A, T, R, C> + 'static,
              A: Operation,
              T: Target
    {
        let id = (TypeId::of::<A>(), TypeId::of::<T>());
        self.methods.insert(id, Box::new(move |a: &Box<Operation>, t: &Target, frame: &Frame<R>, encoder: &mut gfx::Encoder<R, C>| {
            let a = a.downcast_ref::<A>().unwrap();
            let t = t.downcast_ref::<T>().unwrap();
            p.apply(a, t, frame, encoder)
        }));
    }

    /// Execute all passes
    pub fn submit<D>(&mut self, frame: &Frame<R>, device: &mut D)
        where D: gfx::Device<Resources=R, CommandBuffer=C>
    {
        for pass in &frame.passes {
            let target = frame.targets.get(&pass.target).unwrap();
            for op in &pass.operations {
                let id = (mopa::Any::get_type_id(&**op), mopa::Any::get_type_id(&**target));
                let method = self.methods.get(&id).expect("No method found, cannot apply operation to target.");
                method(op, &**target, &frame, &mut self.command_buffer);
            }
        }
        self.command_buffer.flush(device);
        device.cleanup();
    }
}

pub struct Fragment<R: gfx::Resources> {
    pub transform: [[f32; 4]; 4],
    pub buffer: gfx::handle::Buffer<R, VertexPosNormal>,
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

impl<R: gfx::Resources> Target for ScreenOutput<R>
    where <R as gfx::Resources>::RenderTargetView: Any,
          <R as gfx::Resources>::Texture: Any,
          <R as gfx::Resources>::DepthStencilView: Any,
          R: 'static
{}

pub struct Scene<R: gfx::Resources> {
    pub fragments: Vec<Fragment<R>>,
    pub lights: Vec<Light>
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}

pub struct RenderPasses {
    pub target: String,
    pub operations: Vec<Box<Operation>>,
}

impl RenderPasses {
    pub fn new(target: String) -> RenderPasses {
        RenderPasses {
            target: target,
            operations: vec![]
        }
    }
}

/// The render job submission
pub struct Frame<R: gfx::Resources> {
    pub passes: Vec<RenderPasses>,
    pub targets: HashMap<String, Box<Target>>,
    pub scenes: HashMap<String, Scene<R>>,
    pub cameras: HashMap<String, Camera>
}

pub struct Clear {
    pub color: [f32; 4]
}
impl Operation for Clear {}

pub struct Wireframe {
    pub camera: String,
    pub scene: String,
}
impl Operation for Wireframe {}

pub struct FlatShading {
    pub camera: String,
    pub scene: String,
}
impl Operation for FlatShading {}


pub trait Operation: mopa::Any {}
mopafy!(Operation);

pub trait Target: mopa::Any {}
mopafy!(Target);

pub trait Method<A, T, R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
          A: Operation,
          T: Target
{
    fn apply(&self, arg: &A, target: &T, scene: &Frame<R>, encoder: &mut gfx::Encoder<R, C>);
}
