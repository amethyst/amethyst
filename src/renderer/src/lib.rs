#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]
//#![deny(missing_docs)]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
#[macro_use]
extern crate mopa;

extern crate glutin;
extern crate cgmath;

/// Contains the included Render Targets
pub mod target;
/// Contains the included Passes
pub mod pass;

use std::any::TypeId;
use std::collections::HashMap;

pub use pass::PassDescription;
pub use target::Target;
pub use pass::Pass;

/// A Renderer manages passes and the execution of the passes
/// over the targets. It only contains the passes, all other
/// data is contained in the `Frame`
pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    command_buffer: gfx::Encoder<R, C>,
    passes: HashMap<(TypeId, TypeId), Box<Fn(&Box<PassDescription>, &Target, &Frame<R>, &mut gfx::Encoder<R, C>)>>
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
            passes: HashMap::new()
        }
    }

    /// Load all known passes
    pub fn load_all<F>(&mut self, factory: &mut F)
        where F: gfx::Factory<R>
    {
        self.add_pass(pass::forward::Clear);
        self.add_pass(pass::forward::DrawNoShading::new(factory));
        self.add_pass(pass::forward::DrawShaded::new(factory));
        self.add_pass(pass::forward::Wireframe::new(factory));

        self.add_pass(pass::deferred::Clear);
        self.add_pass(pass::deferred::DrawPass::new(factory));
        self.add_pass(pass::deferred::BlitLayer::new(factory));
        self.add_pass(pass::deferred::LightingPass::new(factory));
    }

    /// Add a pass to the table of available passes
    pub fn add_pass<A, T, P>(&mut self, p: P)
        where P: Pass<R, Arg=A, Target=T> + 'static,
              A: PassDescription,
              T: Target
    {
        let id = (TypeId::of::<A>(), TypeId::of::<T>());
        self.passes.insert(id, Box::new(move |a: &Box<PassDescription>, t: &Target, frame: &Frame<R>, encoder: &mut gfx::Encoder<R, C>| {
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
            for desc in &layer.passes {
                let id = (mopa::Any::get_type_id(&**desc), mopa::Any::get_type_id(&**fb));
                if let Some(pass)= self.passes.get(&id) {
                    pass(desc, &**fb, &frame, &mut self.command_buffer);
                } else{
                    panic!("No pass implementation found for target={}, pass={:?}", layer.target, desc);
                }
            }
        }
        self.command_buffer.flush(device);
        device.cleanup();
    }
}

/// A fragment is the most basic drawable element
pub struct Fragment<R: gfx::Resources> {
    /// The transform matrix to apply to the matrix, this
    /// is sometimes refereed to as the model matrix
    pub transform: [[f32; 4]; 4],
    /// The vertex buffer
    pub buffer: gfx::handle::Buffer<R, VertexPosNormal>,
    /// A slice of the above vertex buffer
    pub slice: gfx::Slice<R>,
    /// ambient color
    pub ka: [f32; 4],
    /// diffuse color
    pub kd: [f32; 4]
}

/// A basic light
pub struct Light {
    /// The XYZ coordinate of the light
    pub center: [f32; 3],
    /// How big the light is radius, lighting
    /// passes are not required to render the light
    /// beyond this radius
    pub radius: f32,

    /// The color of light emitted
    pub color: [f32; 4],
    /// constant, propagation means no falloff of the light
    /// emission from distance. Useful for the sun.
    pub propagation_constant: f32,
    /// linear level drops
    pub propagation_linear: f32,
    /// cubic light level drop
    pub propagation_r_square: f32,
}

/// A scene is a collection of fragments and
/// lights that make up the scene.
pub struct Scene<R: gfx::Resources> {
    /// A list of fragments
    pub fragments: Vec<Fragment<R>>,
    /// A list of lights
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


/// Contains the transforms for a Camera
#[derive(Copy, Clone)]
pub struct Camera {
    /// A projection matrix
    pub projection: [[f32; 4]; 4],
    /// A view matrix
    pub view: [[f32; 4]; 4],
}

/// A layer is comprised of a Render target and
/// a list of passes
pub struct Layer {
    /// The render target, looked up  by name during the Frame
    /// submission.
    pub target: String,
    /// A list of passes to be executed in order to build
    /// up the target with the scene's data.
    pub passes: Vec<Box<PassDescription>>,
}

impl Layer {
    /// Create a new pass with that will target the supplied
    /// Target reference, The Layer will be initialized with the suppled
    /// list of passes.
    pub fn new<A>(target: A, passes: Vec<Box<PassDescription>>) -> Layer
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
    /// the layers to be processed
    pub layers: Vec<Layer>,
    /// collection of render targets. A target may be
    /// a source or a sink for a `Pass`
    pub targets: HashMap<String, Box<Target>>,
    /// Collection of scenes, having multiple scenes
    /// allows for selection of different fragments
    /// by different passes
    pub scenes: HashMap<String, Scene<R>>,
    /// Collection of Cameras owned by the scene
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
