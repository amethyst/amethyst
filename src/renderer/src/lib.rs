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

pub mod framebuffer;
pub mod pass;
pub mod method;

use std::any::TypeId;
use std::collections::HashMap;

pub use pass::Pass;
pub use framebuffer::Framebuffer;
pub use method::Method;

pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    command_buffer: gfx::Encoder<R, C>,
    methods: HashMap<(TypeId, TypeId), Box<Fn(&Box<Pass>, &Framebuffer, &Frame<R>, &mut gfx::Encoder<R, C>)>>
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
        self.add_method(method::forward::Clear);
        self.add_method(method::forward::DrawNoShading::new(factory));
        self.add_method(method::forward::Wireframe::new(factory));

        self.add_method(method::deferred::Clear);
        self.add_method(method::deferred::DrawMethod::new(factory));
        self.add_method(method::deferred::BlitLayer::new(factory));
        self.add_method(method::deferred::LightingMethod::new(factory));
    }

    /// Add a method to the table of available methods
    pub fn add_method<A, T, P>(&mut self, p: P)
        where P: Method<A, T, R, C> + 'static,
              A: Pass,
              T: Framebuffer
    {
        let id = (TypeId::of::<A>(), TypeId::of::<T>());
        self.methods.insert(id, Box::new(move |a: &Box<Pass>, t: &Framebuffer, frame: &Frame<R>, encoder: &mut gfx::Encoder<R, C>| {
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
            let fb = frame.framebuffers.get(&pass.target).unwrap();
            for op in &pass.passes {
                let id = (mopa::Any::get_type_id(&**op), mopa::Any::get_type_id(&**fb));
                let method = self.methods.get(&id).expect("No method found, cannot apply passes to target.");
                method(op, &**fb, &frame, &mut self.command_buffer);
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
    pub passes: Vec<Box<Pass>>,
}

impl RenderPasses {
    pub fn new<A>(target: A, passes: Vec<Box<Pass>>) -> RenderPasses
        where String: From<A>
    {
        RenderPasses {
            target: String::from(target),
            passes: passes
        }
    }
}

/// The render job submission
pub struct Frame<R: gfx::Resources> {
    pub passes: Vec<RenderPasses>,
    pub framebuffers: HashMap<String, Box<Framebuffer>>,
    pub scenes: HashMap<String, Scene<R>>,
    pub cameras: HashMap<String, Camera>
}
