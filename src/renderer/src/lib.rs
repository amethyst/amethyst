#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]
// #![deny(missing_docs)]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
#[macro_use]
extern crate mopa;

extern crate glutin;
extern crate cgmath;

extern crate amethyst_ecs;

/// Contains the included Render Targets
pub mod target;
/// Contains the included Passes
pub mod pass;

use amethyst_ecs::{Component, VecStorage};

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
    passes: HashMap<(TypeId, TypeId),
                    Box<Fn(&Box<PassDescription>,
                           &Target,
                           &Pipeline,
                           &Scene<R>,
                           &mut gfx::Encoder<R, C>)>>,
}

// placeholder
gfx_vertex_struct!(VertexPosNormal {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Normal",
    tex_coord: [f32; 2] = "a_TexCoord",
});

impl<R, C> Renderer<R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    /// Create a new Render pipeline
    pub fn new(combuf: C) -> Renderer<R, C> {
        Renderer {
            command_buffer: combuf.into(),
            passes: HashMap::new(),
        }
    }

    /// Load all known passes
    pub fn load_all<F>(&mut self, factory: &mut F)
        where F: gfx::Factory<R>
    {
        self.add_pass(pass::forward::Clear);
        self.add_pass(pass::forward::DrawFlat::new(factory));
        self.add_pass(pass::forward::DrawShaded::new(factory));
        self.add_pass(pass::forward::Wireframe::new(factory));

        self.add_pass(pass::deferred::Clear);
        self.add_pass(pass::deferred::DrawPass::new(factory));
        self.add_pass(pass::deferred::DepthPass::new(factory));
        self.add_pass(pass::deferred::BlitLayer::new(factory));
        self.add_pass(pass::deferred::LightingPass::new(factory));
    }

    /// Add a pass to the table of available passes
    pub fn add_pass<A, T, P>(&mut self, p: P)
        where P: Pass<R, Arg = A, Target = T> + 'static,
              A: PassDescription,
              T: Target
    {
        let id = (TypeId::of::<A>(), TypeId::of::<T>());
        self.passes.insert(id,
                           Box::new(move |a: &Box<PassDescription>, t: &Target, pipeline: &Pipeline, scene: &Scene<R>, encoder: &mut gfx::Encoder<R, C>| {
                               let a = a.downcast_ref::<A>().unwrap();
                               let t = t.downcast_ref::<T>().unwrap();
                               p.apply(a, t, pipeline, scene, encoder)
                           }));
    }

    /// Execute all passes
    pub fn submit<D>(&mut self, pipeline: &Pipeline, scene: &Scene<R>, device: &mut D)
        where D: gfx::Device<Resources = R, CommandBuffer = C>
    {
        for layer in &pipeline.layers {
            let fb = pipeline.targets.get(&layer.target).unwrap();
            for desc in &layer.passes {
                let id = (mopa::Any::get_type_id(&**desc), mopa::Any::get_type_id(&**fb));
                if let Some(pass) = self.passes.get(&id) {
                    pass(desc, &**fb, &pipeline, &scene, &mut self.command_buffer);
                } else {
                    panic!("No pass implementation found for target={}, pass={:?}",
                           layer.target,
                           desc);
                }
            }
        }
        self.command_buffer.flush(device);
        device.cleanup();
    }
}

/// holds a 1x1 texture that can be used to store constant colors
#[derive(Clone)]
pub struct ConstantColorTexture<R: gfx::Resources> {
    texture: gfx::handle::Texture<R, gfx::format::R8_G8_B8_A8>,
    view: gfx::handle::ShaderResourceView<R, [f32; 4]>,
}

impl<R: gfx::Resources> ConstantColorTexture<R> {
    /// Create a texture buffer
    pub fn new<F>(factory: &mut F) -> ConstantColorTexture<R>
        where F: gfx::Factory<R>
    {
        let kind = gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single);
        let text = factory.create_texture::<gfx::format::R8_G8_B8_A8>(kind,
                                                        1,
                                                        gfx::SHADER_RESOURCE,
                                                        gfx::Usage::Dynamic,
                                                        Some(gfx::format::ChannelType::Unorm))
            .unwrap();
        let levels = (0, text.get_info().levels - 1);
        let view = factory.view_texture_as_shader_resource::<gfx::format::Rgba8>(&text, levels, gfx::format::Swizzle::new())
            .unwrap();
        ConstantColorTexture {
            texture: text,
            view: view,
        }
    }
}

#[derive(Clone)]
pub enum Texture<R: gfx::Resources> {
    Constant([f32; 4]),
    Texture(gfx::handle::ShaderResourceView<R, [f32; 4]>),
}

impl<R: gfx::Resources> Texture<R> {
    pub fn to_view<C>(&self, texture: &ConstantColorTexture<R>, encoder: &mut gfx::Encoder<R, C>) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
        where C: gfx::CommandBuffer<R>
    {
        match self {
            &Texture::Constant(color) => {
                let color: [[u8; 4]; 1] = [[
                    (color[0] * 255.) as u8,
                    (color[1] * 255.) as u8,
                    (color[2] * 255.) as u8,
                    (color[3] * 255.) as u8,
                ]];

                encoder.update_texture::<_, gfx::format::Rgba8>(
                    &texture.texture,
                    None,
                    texture.texture.get_info().to_image_info(0),
                    &color[..]
                ).unwrap();

                texture.view.clone()
            }
            &Texture::Texture(ref tex) => tex.clone(),
        }
    }
}

/// A fragment is the most basic drawable element
#[derive(Clone)]
pub struct Fragment<R: gfx::Resources> {
    /// The transform matrix to apply to the matrix, this
    /// is sometimes referred to as the model matrix
    pub transform: [[f32; 4]; 4],
    /// The vertex buffer
    pub buffer: gfx::handle::Buffer<R, VertexPosNormal>,
    /// A slice of the above vertex buffer
    pub slice: gfx::Slice<R>,
    /// ambient color
    pub ka: Texture<R>,
    /// diffuse color
    pub kd: Texture<R>,
    /// specular color
    pub ks: Texture<R>,
    /// specular exponent
    pub ns: f32,
}


/// Represents a point light. Lighting calculations are based off of
/// the Frostbite engine's lighting, which is explained in detail here:
/// http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr.pdf
/// The particular equation used for our calculations is Eq. 26, and
/// the `PointLight` properties below map as `I -> intensity`,
/// `radius -> lightRadius`, and `n -> smoothness`.
#[derive(Copy, Clone)]
pub struct PointLight {
    /// The XYZ coordinate of this light
    pub center: [f32; 3],

    /// The color of light emitted
    pub color: [f32; 4],

    /// The brightness of this light
    pub intensity: f32,

    /// What distance this light's effects will be clamped to
    pub radius: f32,

    /// How smooth the transition from light to dark is at the edge
    /// of this light's radius
    pub smoothness: f32,
}

impl Default for PointLight {
    fn default() -> PointLight {
        PointLight {
            color: [1.0, 1.0, 1.0, 1.0],
            center: [0.0, 0.0, 0.0],
            intensity: 10.0,
            radius: 10.0,
            smoothness: 4.0,
        }
    }
}

impl Component for PointLight {
    type Storage = VecStorage<PointLight>;
}


/// Represents a directional light
#[derive(Copy, Clone)]
pub struct DirectionalLight {
    /// The color of light emitted
    pub color: [f32; 4],

    /// Which direction this light shines towards
    pub direction: [f32; 3],
}

impl Default for DirectionalLight {
    fn default() -> DirectionalLight {
        DirectionalLight {
            color: [1.0; 4],
            direction: [-1.0; 3],
        }
    }
}

impl Component for DirectionalLight {
    type Storage = VecStorage<DirectionalLight>;
}


/// Represents an ambient light
pub struct AmbientLight {
    // How powerful the ambient light factor is
    pub power: f32,
}

impl Default for AmbientLight {
    fn default() -> AmbientLight {
        AmbientLight {
            power: 0.01,
        }
    }
}

/// A scene is a collection of fragments and
/// lights that make up the scene.
#[derive(Clone)]
pub struct Scene<R: gfx::Resources> {
    /// A list of fragments
    pub fragments: Vec<Fragment<R>>,

    /// A list of point lights
    pub point_lights: Vec<PointLight>,

    /// A list of directional lights
    pub directional_lights: Vec<DirectionalLight>,

    /// The ambient light factor
    pub ambient_light: f32,

    /// A camera used to render this scene
    pub camera: Camera,
}

impl<R: gfx::Resources> Scene<R> {
    /// Create an empty scene
    pub fn new(camera: Camera) -> Scene<R> {
        Scene {
            fragments: vec![],
            point_lights: vec![],
            directional_lights: vec![],
            ambient_light: 0.01,
            camera: camera,
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

impl Camera {
    pub fn new(proj: [[f32; 4]; 4], view: [[f32; 4]; 4]) -> Camera {
        Camera {
            projection: proj,
            view: view,
        }
    }

    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
        cgmath::perspective(cgmath::Deg(fov), aspect, near, far).into()
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
        cgmath::ortho(left, right, bottom, top, near, far).into()
    }

    pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
        use cgmath::{Point3, Vector3, Matrix4, Transform};
        let view: Matrix4<f32> = Transform::look_at(Point3::new(eye[0], eye[1], eye[2]),
                                                    Point3::new(target[0], target[1], target[2]),
                                                    Vector3::new(up[0], up[1], up[2]));
        view.into()
    }
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
            passes: passes,
        }
    }
}

/// The render job submission
pub struct Pipeline {
    /// the layers to be processed
    pub layers: Vec<Layer>,
    /// collection of render targets. A target may be
    /// a source or a sink for a `Pass`
    pub targets: HashMap<String, Box<Target>>,
}

impl Pipeline {
    /// Create an empty Pipeline
    pub fn new() -> Pipeline {
        Pipeline {
            layers: vec![],
            targets: HashMap::new(),
        }
    }
}
