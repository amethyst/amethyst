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

pub mod target;
pub mod pass;
pub mod method;

use std::any::TypeId;
use std::collections::HashMap;

pub use pass::Pass;
pub use target::Target;
pub use method::Method;

pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    command_buffer: gfx::Encoder<R, C>,
    methods: HashMap<(TypeId, TypeId), Box<Fn(&Box<Pass>, &Target, &Frame<R>, &mut gfx::Encoder<R, C>)>>
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
        where P: Method<R, Arg=A, Target=T> + 'static,
              A: Pass,
              T: Target
    {
        let id = (TypeId::of::<A>(), TypeId::of::<T>());
        self.methods.insert(id, Box::new(move |a: &Box<Pass>, t: &Target, frame: &Frame<R>, encoder: &mut gfx::Encoder<R, C>| {
            let a = a.downcast_ref::<A>().unwrap();
            let t = t.downcast_ref::<T>().unwrap();
            p.apply(a, t, frame, encoder)
        }));
    }

    /// Execute all passes
    pub fn submit<D>(&mut self, frame: &Frame<R>, device: &mut D)
        where D: gfx::Device<Resources=R, CommandBuffer=C>
    {
        for layer in &frame.layers {
            let fb = frame.targets.get(&layer.target).unwrap();
            for pass in &layer.passes {
                let id = (mopa::Any::get_type_id(&**pass), mopa::Any::get_type_id(&**fb));
                let method = self.methods.get(&id).expect("No method found, cannot apply passes to target.");
                method(pass, &**fb, &frame, &mut self.command_buffer);
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

impl<R: gfx::Resources> Scene<R> {
    /// Create an empty scene
    pub fn new() -> Scene<R> {
        Scene{
            fragments: vec![],
            lights: vec![]
        }
    }
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
}

pub struct Layer {
    pub target: String,
    pub passes: Vec<Box<Pass>>,
}

impl Layer {
    pub fn new<A>(target: A, passes: Vec<Box<Pass>>) -> Layer
        where String: From<A>
    {
        Layer {
            target: String::from(target),
            passes: passes
        }
    }
}

/// The render job submission
pub struct Frame<R: gfx::Resources> {
    pub layers: Vec<Layer>,
    pub targets: HashMap<String, Box<Target>>,
    pub scenes: HashMap<String, Scene<R>>,
    pub cameras: HashMap<String, Camera>
}

impl<R: gfx::Resources> Frame<R> {
    /// Create an empty Frame
    pub fn new() -> Frame<R> {
        Frame {
            layers: vec![],
            targets: HashMap::new(),
            scenes: HashMap::new(),
            cameras: HashMap::new()
        }
    }
}