#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
#[macro_use]
extern crate mopa;

extern crate cgmath;
extern crate glutin;
extern crate specs;

pub mod target;
pub mod pass;

use specs::{Component, VecStorage};
use std::any::TypeId;
use std::collections::HashMap;

pub use pass::Pass;
pub use pass::PassDescription;
pub use target::Target;

/// Manages passes and the execution of the passes over the targets. It only
/// contains the passes, all other data is contained in the `Frame`.
pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    cmd_buf: gfx::Encoder<R, C>,
    passes: HashMap<(TypeId, TypeId),
                    Box<Fn(&Box<PassDescription>,
                           &Target,
                           &Pipeline,
                           &Scene<R>,
                           &mut gfx::Encoder<R, C>)>>,
}

/// NOTE: This is just a placeholder!
#[allow(missing_docs)]
gfx_vertex_struct!(VertexPosNormal {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Normal",
    tex_coord: [f32; 2] = "a_TexCoord",
});

impl<R, C> Renderer<R, C>
    where R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    /// Creates a new renderer with the given command buffer.
    pub fn new(cmd_buf: C) -> Renderer<R, C> {
        Renderer {
            cmd_buf: cmd_buf.into(),
            passes: HashMap::new(),
        }
    }

    /// Load all known passes into the renderer.
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

    /// Add a pass to the table of available passes.
    pub fn add_pass<A, T, P>(&mut self, p: P)
        where P: Pass<R, Arg = A, Target = T> + 'static,
              A: PassDescription,
              T: Target
    {
        let id = (TypeId::of::<A>(), TypeId::of::<T>());
        self.passes.insert(id,
                           Box::new(move |a: &Box<PassDescription>,
                                          t: &Target,
                                          pipeline: &Pipeline,
                                          scene: &Scene<R>,
                                          encoder: &mut gfx::Encoder<R, C>| {
                               let a = a.downcast_ref::<A>().unwrap();
                               let t = t.downcast_ref::<T>().unwrap();
                               p.apply(a, t, pipeline, scene, encoder)
                           }));
    }

    /// Execute all passes and draw the frame.
    pub fn submit<D>(&mut self, pipe: &Pipeline, scene: &Scene<R>, device: &mut D)
        where D: gfx::Device<Resources = R, CommandBuffer = C>
    {
        for layer in &pipe.layers {
            let fb = pipe.targets.get(&layer.target).unwrap();
            for desc in &layer.passes {
                let id = (mopa::Any::get_type_id(&**desc), mopa::Any::get_type_id(&**fb));
                if let Some(pass) = self.passes.get(&id) {
                    pass(desc, &**fb, &pipe, &scene, &mut self.cmd_buf);
                } else {
                    panic!("No pass implementation found for target={}, pass={:?}",
                           layer.target,
                           desc);
                }
            }
        }

        self.cmd_buf.flush(device);
        device.cleanup();
    }
}

/// Tiny 1x1 texture that can be used to store constant colors.
#[derive(Clone)]
pub struct ConstantColorTexture<R: gfx::Resources> {
    /// Handle to the texture resource.
    texture: gfx::handle::Texture<R, gfx::format::R8_G8_B8_A8>,
    /// Immutable view of the texture resource above.
    view: gfx::handle::ShaderResourceView<R, [f32; 4]>,
}

impl<R: gfx::Resources> ConstantColorTexture<R> {
    /// Create a new `ConstantColorTexture` from the given factory.
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
        let view = factory.view_texture_as_shader_resource::<gfx::format::Rgba8>(&text,
                                                                   levels,
                                                                   gfx::format::Swizzle::new())
            .unwrap();
        ConstantColorTexture {
            texture: text,
            view: view,
        }
    }
}

/// A renderable texture resource.
#[derive(Clone)]
pub enum Texture<R: gfx::Resources> {
    /// A texture with one constant RGBA color value.
    Constant([f32; 4]),
    /// Handle to a slice of texture memory.
    Texture(gfx::handle::ShaderResourceView<R, [f32; 4]>),
}

impl<R: gfx::Resources> Texture<R> {
    /// Takes the given constant color texture and, using the encoder, returns a
    /// slice of texture memory.
    pub fn to_view<C>(&self,
                      texture: &ConstantColorTexture<R>,
                      encoder: &mut gfx::Encoder<R, C>)
                      -> gfx::handle::ShaderResourceView<R, [f32; 4]>
        where C: gfx::CommandBuffer<R>
    {
        match *self {
            Texture::Constant(ref color) => {
                let color: [[u8; 4]; 1] = [[(color[0] * 255.) as u8,
                                            (color[1] * 255.) as u8,
                                            (color[2] * 255.) as u8,
                                            (color[3] * 255.) as u8]];

                encoder.update_texture::<_, gfx::format::Rgba8>(&texture.texture,
                                                             None,
                                                             texture.texture
                                                                 .get_info()
                                                                 .to_image_info(0),
                                                             &color[..])
                    .unwrap();

                texture.view.clone()
            }
            Texture::Texture(ref tex) => tex.clone(),
        }
    }
}

/// The most basic drawable element.
#[derive(Clone)]
pub struct Fragment<R: gfx::Resources> {
    /// The transform matrix to apply to the matrix. This is sometimes referred
    /// to as the model matrix. FIXME: Wording needs clarification.
    pub transform: [[f32; 4]; 4],
    /// Vertex buffer associated with this fragment.
    pub buffer: gfx::handle::Buffer<R, VertexPosNormal>,
    /// A slice of the vertex buffer above.
    pub slice: gfx::Slice<R>,
    /// The ambient color.
    pub ka: Texture<R>,
    /// The diffuse color.
    pub kd: Texture<R>,
    /// The specular color.
    pub ks: Texture<R>,
    /// The pecular exponent.
    pub ns: f32,
}

/// A point light source.
///
/// Lighting calculations are based off of the Frostbite engine's lighting,
/// which is explained in detail here in [this presentation][fb]. The particular
/// equation used for our calculations is Eq. 26, and the `PointLight`
/// properties below map like so:
///
/// * *I* = `intensity`
/// * *radius* = `radius`
/// * *n* = `smoothness`
///
/// [fb]: http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr.pdf
#[derive(Copy, Clone, Debug)]
pub struct PointLight {
    /// Coordinates of the light source in three dimensional space.
    pub center: [f32; 3],
    /// Color of the light.
    pub color: [f32; 4],
    /// Brightness of the light source.
    pub intensity: f32,
    /// Maximum radius of the point light's affected area.
    pub radius: f32,
    /// Smoothness of the light-to-dark transition from the center to the
    /// radius.
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

/// A directional light source.
#[derive(Clone, Copy, Debug)]
pub struct DirectionalLight {
    /// Color of the light.
    pub color: [f32; 4],
    /// Direction that the light is pointing.
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

/// An ambient light source.
#[derive(Clone, Copy, Debug)]
pub struct AmbientLight {
    /// Intensity of the light.
    pub power: f32,
}

impl Default for AmbientLight {
    fn default() -> AmbientLight {
        AmbientLight { power: 0.01 }
    }
}

/// Collection of fragments and lights that make up the scene.
#[derive(Clone)]
pub struct Scene<R: gfx::Resources> {
    /// List of renderable fragments.
    pub fragments: Vec<Fragment<R>>,
    /// List of point lights.
    pub point_lights: Vec<PointLight>,
    /// List of directional lights.
    pub directional_lights: Vec<DirectionalLight>,
    /// Ambient light factor.
    pub ambient_light: f32,
    /// A camera used to render this scene
    pub camera: Camera,
}

impl<R: gfx::Resources> Scene<R> {
    /// Creates an empty scene with the given camera.
    pub fn new(camera: Camera) -> Scene<R> {
        Scene {
            fragments: Vec::new(),
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            ambient_light: 0.01,
            camera: camera,
        }
    }
}

/// Contains the graphical transforms for a camera.
#[derive(Copy, Clone)]
pub struct Camera {
    /// Graphical projection matrix.
    pub proj: [[f32; 4]; 4],
    /// The view matrix.
    pub view: [[f32; 4]; 4],
}

impl Camera {
    /// Creates a new camera with the given projection and view matrices.
    pub fn new(proj: [[f32; 4]; 4], view: [[f32; 4]; 4]) -> Camera {
        Camera {
            proj: proj,
            view: view,
        }
    }

    /// Returns a realistic perspective projection matrix.
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
        cgmath::perspective(cgmath::Deg(fov), aspect, near, far).into()
    }

    /// Returns an orthographic projection matrix..
    pub fn orthographic(left: f32,
                        right: f32,
                        bottom: f32,
                        top: f32,
                        near: f32,
                        far: f32)
                        -> [[f32; 4]; 4] {
        cgmath::ortho(left, right, bottom, top, near, far).into()
    }

    /// Returns a 4x4 view matrix.
    pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
        use cgmath::{Point3, Vector3, Matrix4, Transform};
        let view: Matrix4<f32> = Transform::look_at(Point3::new(eye[0], eye[1], eye[2]),
                                                    Point3::new(target[0], target[1], target[2]),
                                                    Vector3::new(up[0], up[1], up[2]));
        view.into()
    }
}

/// A stackable image layer.
///
/// Layers contain a list of passes which are used to render a final image which
/// is then composited onto a render target. They are especially useful for
/// postprocessing, e.g. applying a fullscreen night vision effect, drawing a
/// HUD (heads-up display) over a rendered scene.
pub struct Layer {
    /// Name of the render target to draw on.
    pub target: String,
    /// Sequence of passes to execute over the render target.
    pub passes: Vec<Box<PassDescription>>,
}

impl Layer {
    /// Creates a new layer with the given list of passes and the name of the
    /// render target
    pub fn new<T>(target: T, passes: Vec<Box<PassDescription>>) -> Layer
        where String: From<T>
    {
        Layer {
            target: String::from(target),
            passes: passes,
        }
    }
}

/// The render job submission
/// Describes the layers and 
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
            layers: Vec::new(),
            targets: HashMap::new(),
        }
    }
}
