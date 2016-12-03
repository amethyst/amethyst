#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]
// #![deny(missing_docs)]

//! High-level rendering engine with multiple backends.

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
#[macro_use]
extern crate mopa;

extern crate glutin;
extern crate cgmath;

extern crate amethyst_ecs;

/// Contains the included Render Targets
pub mod target;
/// Contains the included Passes
pub mod pass;

use self::amethyst_ecs::{Allocator, Entity};

use std::any::TypeId;
use std::collections::{HashMap};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::cell::RefCell;

use gfx::{buffer, format, handle, mapping, texture, Factory};
use gfx_core::handle::Producer;
use gfx_core::memory::Typed;

pub use pass::PassDescription;
pub use target::Target;
pub use pass::Pass;

/// A Renderer manages passes and the execution of the passes
/// over the targets. It only contains the passes, all other
/// data is contained in the `Frame`
pub struct Renderer<R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>> {
    command_buffer: gfx::Encoder<R, C>,
    passes: HashMap<(TypeId, TypeId),
                    Box<Fn(&mut ResourceCache<R>,
                           &Box<PassDescription>,
                           &Target,
                           &Frame,
                           &mut gfx::Encoder<R, C>)>>,
    factory_sink: GpuFactorySink<R, F>,
    resources: ResourceCache<R>,
}

pub struct ResourceCache<R: gfx::Resources> {
    srvs: HashMap<Entity, gfx::handle::RawShaderResourceView<R>>,
    buffers: HashMap<Entity, gfx::handle::RawBuffer<R>>,

    handles: handle::Manager<Resources>,
}

impl<R: gfx::Resources> ResourceCache<R> {
    pub fn new() -> ResourceCache<R> {
        ResourceCache {
            srvs: HashMap::new(),
            buffers: HashMap::new(),
            handles: handle::Manager::new(),
        }
    }

    pub fn get_srv(&mut self, id: &gfx::handle::RawShaderResourceView<Resources>) -> gfx::handle::RawShaderResourceView<R> {
        let id = self.handles.ref_srv(id);
        self.srvs.get(id).unwrap().clone()
    }

    pub fn get_slice(&mut self, slice: &gfx::Slice<Resources>) -> gfx::Slice<R> {
        use gfx::IndexBuffer;
        let buffer = match slice.buffer {
            IndexBuffer::Auto => IndexBuffer::Auto,
            IndexBuffer::Index16(ref buf) => {
                let id = self.handles.ref_buffer(buf.raw());
                IndexBuffer::Index16(gfx::handle::Buffer::new(self.buffers.get(id).unwrap().clone()))
            }
            IndexBuffer::Index32(ref buf) => {
                let id = self.handles.ref_buffer(buf.raw());
                IndexBuffer::Index32(gfx::handle::Buffer::new(self.buffers.get(id).unwrap().clone()))
            }
        };

        gfx::Slice {
            start: slice.start,
            end: slice.end,
            base_vertex: slice.base_vertex,
            instances: slice.instances,
            buffer: buffer,
        }
    }

    pub fn get_buffer(&mut self, id: &gfx::handle::RawBuffer<Resources>) -> gfx::handle::RawBuffer<R> {
        let id = self.handles.ref_buffer(id);
        self.buffers.get(id).unwrap().clone()
    }
}

pub enum GpuFactoryCmd<R: gfx::Resources> {
    CmdCreateBuffer(handle::RawBuffer<Resources>),
    CmdCreateImmutableBuffer(handle::RawBuffer<Resources>, Vec<u8>),
    CmdCreateTexture(handle::RawShaderResourceView<Resources>, Option<Vec<u8>>),

    CmdAddBuffer(handle::RawBuffer<R>),
    CmdAddTexture(handle::RawShaderResourceView<R>),
}

pub struct GpuFactorySink<R: gfx::Resources, F: gfx::Factory<R>> {
    receiver: Receiver<(Entity, GpuFactoryCmd<R>)>,
    factory: F,
    _marker: std::marker::PhantomData<R>,
}

pub trait ParallelFactory<R: gfx::Resources> {
    type Factory;

    fn create_buffer_raw(factory: &mut Self::Factory, buffer: &handle::RawBuffer<Resources>) -> Result<GpuFactoryCmd<R>, buffer::CreationError>;
    fn create_buffer_immutable_raw(factory: &mut Self::Factory, buffer: &handle::RawBuffer<Resources>, data: &[u8])
                               -> Result<GpuFactoryCmd<R>, buffer::CreationError>;

}

impl ParallelFactory<gfx_device_gl::Resources> for gfx_device_gl::Resources {
    type Factory = ();
    fn create_buffer_raw(factory: &mut Self::Factory, buffer: &handle::RawBuffer<Resources>) -> Result<GpuFactoryCmd<gfx_device_gl::Resources>, buffer::CreationError> {
        Ok(GpuFactoryCmd::CmdCreateBuffer(buffer.clone()))
    }
    fn create_buffer_immutable_raw(factory: &mut Self::Factory, buffer: &handle::RawBuffer<Resources>, data: &[u8])
                               -> Result<GpuFactoryCmd<gfx_device_gl::Resources>, buffer::CreationError>
    {
        Ok(GpuFactoryCmd::CmdCreateImmutableBuffer(buffer.clone(), data.to_vec()))
    }
}

pub struct GpuFactoryStream<R: gfx::Resources + ParallelFactory<R>> {
    sender: Sender<(Entity, GpuFactoryCmd<R>)>,
    allocator: Arc<Allocator>,
    handles: Arc<RefCell<handle::Manager<Resources>>>,
    parallel_factory: R::Factory,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Resources {}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fence;
impl gfx_core::Fence for Fence {
    fn wait(&self) { unimplemented!() }
}

#[derive(Debug)]
pub struct MappingGate;
impl mapping::Gate<Resources> for MappingGate {
    unsafe fn set<T>(&self, index: usize, val: T) { unimplemented!() }
    unsafe fn slice<'a, 'b, T>(&'a self, len: usize) -> &'b [T] { unimplemented!() }
    unsafe fn mut_slice<'a, 'b, T>(&'a self, len: usize) -> &'b mut [T] { unimplemented!() }
}

impl gfx::Resources for Resources {
    type Buffer              = Entity;
    type Shader              = Entity;
    type Program             = Entity;
    type PipelineStateObject = Entity;
    type Texture             = Entity;
    type RenderTargetView    = Entity;
    type DepthStencilView    = Entity;
    type ShaderResourceView  = Entity;
    type UnorderedAccessView = ();
    type Sampler             = Entity;
    type Fence               = Fence;
    type Mapping             = MappingGate;
}

impl<R: gfx::Resources + ParallelFactory<R>> Factory<Resources> for GpuFactoryStream<R> {
    fn get_capabilities(&self) -> &gfx_core::Capabilities {
        unimplemented!()
    }

    fn create_buffer_raw(&mut self, info: buffer::Info) -> Result<handle::RawBuffer<Resources>, buffer::CreationError> {
        let name = self.allocator.allocate_atomic();
        let buffer = self.handles.borrow_mut().make_buffer(name, info);
        R::create_buffer_raw(&mut self.parallel_factory, &buffer).map(|cmd| {
            self.sender.send((name, cmd));
            buffer
        })
    }
    
    fn create_buffer_immutable_raw(&mut self, data: &[u8], stride: usize, role: buffer::Role, bind: gfx_core::memory::Bind)
                               -> Result<handle::RawBuffer<Resources>, buffer::CreationError> {
        let name = self.allocator.allocate_atomic();
        let info = buffer::Info {
            role: role,
            usage: gfx_core::memory::Usage::Immutable,
            bind: bind,
            size: data.len(),
            stride: stride,
        };
        let buffer = self.handles.borrow_mut().make_buffer(name, info);
        R::create_buffer_immutable_raw(&mut self.parallel_factory, &buffer, data).map(|cmd| {
            self.sender.send((name, cmd));
            buffer
        })
    }

    fn create_shader(&mut self, _stage: gfx_core::shade::Stage, _code: &[u8])
                     -> Result<handle::Shader<Resources>, gfx_core::shade::CreateShaderError> {
        unimplemented!()
    }

    fn create_program(&mut self, _shader_set: &gfx_core::ShaderSet<Resources>)
                      -> Result<handle::Program<Resources>, gfx_core::shade::CreateProgramError> {
        unimplemented!()
    }

    fn create_pipeline_state_raw(&mut self, _program: &handle::Program<Resources>, _desc: &gfx_core::pso::Descriptor)
                                 -> Result<handle::RawPipelineState<Resources>, gfx_core::pso::CreationError> {
        unimplemented!()
    }

    fn create_texture_raw(&mut self, _desc: texture::Info, _hint: Option<format::ChannelType>, _data_opt: Option<&[&[u8]]>)
                          -> Result<handle::RawTexture<Resources>, texture::CreationError> {
        unimplemented!()
    }

    fn view_buffer_as_shader_resource_raw(&mut self, _hbuf: &handle::RawBuffer<Resources>)
                                      -> Result<handle::RawShaderResourceView<Resources>, gfx_core::factory::ResourceViewError> {
        unimplemented!()
    }

    fn view_buffer_as_unordered_access_raw(&mut self, _hbuf: &handle::RawBuffer<Resources>)
                                       -> Result<handle::RawUnorderedAccessView<Resources>, gfx_core::factory::ResourceViewError> {
        unimplemented!()
    }

    fn view_texture_as_shader_resource_raw(&mut self, _htex: &handle::RawTexture<Resources>, _desc: texture::ResourceDesc)
                                       -> Result<handle::RawShaderResourceView<Resources>, gfx_core::factory::ResourceViewError> {
        unimplemented!()
    }

    fn view_texture_as_unordered_access_raw(&mut self, _htex: &handle::RawTexture<Resources>)
                                        -> Result<handle::RawUnorderedAccessView<Resources>, gfx_core::factory::ResourceViewError> {
        unimplemented!()
    }

    fn view_texture_as_render_target_raw(&mut self, _htex: &handle::RawTexture<Resources>, _desc: texture::RenderDesc)
                                         -> Result<handle::RawRenderTargetView<Resources>, gfx_core::factory::TargetViewError> {
        unimplemented!()
    }

    fn view_texture_as_depth_stencil_raw(&mut self, _htex: &handle::RawTexture<Resources>, _desc: texture::DepthStencilDesc)
                                         -> Result<handle::RawDepthStencilView<Resources>, gfx_core::factory::TargetViewError> {
        unimplemented!()
    }

    fn create_sampler(&mut self, _info: texture::SamplerInfo) -> handle::Sampler<Resources> {
        unimplemented!()
    }

    fn map_buffer_raw(&mut self, _buf: &handle::RawBuffer<Resources>, _access: gfx_core::memory::Access)
                      -> Result<handle::RawMapping<Resources>, mapping::Error> {
        unimplemented!()
    }

    fn map_buffer_readable<T: Copy>(&mut self, _buf: &handle::Buffer<Resources, T>)
                                    -> Result<mapping::Readable<Resources, T>, mapping::Error> {
        unimplemented!()
    }

    fn map_buffer_writable<T: Copy>(&mut self, _buf: &handle::Buffer<Resources, T>)
                                    -> Result<mapping::Writable<Resources, T>, mapping::Error> {
        unimplemented!()
    }

    fn map_buffer_rw<T: Copy>(&mut self, _buf: &handle::Buffer<Resources, T>)
                              -> Result<mapping::RWable<Resources, T>, mapping::Error> {
        unimplemented!()
    }
}

// placeholder
gfx_vertex_struct!(VertexPosNormal {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Normal",
    tex_coord: [f32; 2] = "a_TexCoord",
});

impl<R, C, F> Renderer<R, C, F>
    where R: gfx::Resources + ParallelFactory<R>,
          C: gfx::CommandBuffer<R>,
          F: gfx::Factory<R>,
{
    /// Create a new Render pipline
    pub fn new(combuf: C, factory: F, parallel_factory: R::Factory) -> (Renderer<R, C, F>, GpuFactoryStream<R>) {
        let (stream, sink) = Self::build_factory(factory, parallel_factory);
        let renderer = Renderer {
            command_buffer: combuf.into(),
            passes: HashMap::new(),
            factory_sink: sink,
            resources: ResourceCache::new(),
        };
        (renderer, stream)
    }

    pub fn update_resources(&mut self) {
        let sink = &mut self.factory_sink;
        loop {
            if let Ok((id, value)) = sink.receiver.try_recv() {
                use self::GpuFactoryCmd::*;
                match value {
                    CmdCreateBuffer(_) => {
                        unimplemented!()
                    }
                    CmdCreateImmutableBuffer(buffer, data) => {
                        let info = buffer.get_info();
                        // TODO(msiglreith): handle error
                        let buffer = sink.factory.create_buffer_immutable_raw(&data, info.stride, info.role, info.bind).unwrap();
                        self.resources.buffers.insert(id, buffer);
                    }
                    CmdCreateTexture(texture, data_opt) => {
                        unimplemented!()
                    }

                    CmdAddBuffer(buffer) => {
                        self.resources.buffers.insert(id, buffer);
                    }
                    CmdAddTexture(texture) => {
                        self.resources.srvs.insert(id, texture);
                    }
                }
            } else {
                return;
            }
        }
    }

    fn build_factory(factory: F, parallel_factory: R::Factory) -> (GpuFactoryStream<R>, GpuFactorySink<R, F>)
        where F: gfx::Factory<R>
    {
        let allocator = Arc::new(Allocator::new());
        let handles = Arc::new(RefCell::new(handle::Manager::new()));
        let (tx, rx) = mpsc::channel();
        let stream = GpuFactoryStream {
            sender: tx,
            allocator: allocator.clone(),
            handles: handles,
            parallel_factory: parallel_factory,
        };
        let sink = GpuFactorySink {
            receiver: rx,
            factory: factory,
            _marker: std::marker::PhantomData,
        };
        (stream, sink)
    }

    /// Load all known passes
    pub fn load_all(&mut self, factory: &mut F)
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
                           Box::new(move |r: &mut ResourceCache<R>, a: &Box<PassDescription>, t: &Target, frame: &Frame, encoder: &mut gfx::Encoder<R, C>| {
                               let a = a.downcast_ref::<A>().unwrap();
                               let t = t.downcast_ref::<T>().unwrap();
                               p.apply(r, a, t, frame, encoder)
                           }));
    }

    /// Execute all passes
    pub fn submit<D>(&mut self, frame: &Frame, device: &mut D)
        where D: gfx::Device<Resources = R, CommandBuffer = C>
    {
        self.update_resources();
        self.resources.handles.clear();
        for layer in &frame.layers {
            let fb = frame.targets.get(&layer.target).unwrap();
            for desc in &layer.passes {
                let id = (mopa::Any::get_type_id(&**desc), mopa::Any::get_type_id(&**fb));
                if let Some(pass) = self.passes.get(&id) {
                    pass(&mut self.resources, desc, &**fb, &frame, &mut self.command_buffer);
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
        let kind = gfx::texture::Kind::D2(1, 1, gfx::texture::AaMode::Single);
        let text = factory.create_texture::<gfx::format::R8_G8_B8_A8>(kind,
                                                        1,
                                                        gfx::SHADER_RESOURCE,
                                                        gfx::memory::Usage::Dynamic,
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

/// A wraper around `Texture` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub enum Texture {
    Constant([f32; 4]),
    Texture(gfx::handle::ShaderResourceView<Resources, [f32; 4]>),
}

impl Texture {
    pub fn to_view<C, R: gfx::Resources>(&self, texture: &ConstantColorTexture<R>, resources: &mut ResourceCache<R>, encoder: &mut gfx::Encoder<R, C>) -> gfx::handle::ShaderResourceView<R, [f32; 4]>
        where C: gfx::CommandBuffer<R>
    {
        match self {
            &Texture::Constant(color) => {
                let color: [[u8; 4]; 1] = [[(color[0] * 255.) as u8, (color[1] * 255.) as u8, (color[2] * 255.) as u8, (color[3] * 255.) as u8]];
                encoder.update_texture::<_, gfx::format::Rgba8>(&texture.texture,
                                                             None,
                                                             texture.texture
                                                                 .get_info()
                                                                 .to_image_info(0),
                                                             &color[..])
                    .unwrap();
                texture.view.clone()
            }
            &Texture::Texture(ref tex) => gfx::handle::ShaderResourceView::new(resources.get_srv(tex.raw())),
        }
    }
}

/// A fragment is the most basic drawable element
pub struct Fragment {
    /// The transform matrix to apply to the matrix, this
    /// is sometimes refereed to as the model matrix
    pub transform: [[f32; 4]; 4],
    /// The vertex buffer
    pub buffer: gfx::handle::Buffer<Resources, VertexPosNormal>,
    /// A slice of the above vertex buffer
    pub slice: gfx::Slice<Resources>,
    /// ambient color
    pub ka: Texture,
    /// diffuse color
    pub kd: Texture,
}

/// A basic light
#[derive(Copy, Clone)]
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
pub struct Scene {
    /// A list of fragments
    pub fragments: Vec<Fragment>,
    /// A list of lights
    pub lights: Vec<Light>,
}

impl Scene {
    /// Create an empty scene
    pub fn new() -> Scene {
        Scene {
            fragments: vec![],
            lights: vec![],
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
pub struct Frame {
    /// the layers to be processed
    pub layers: Vec<Layer>,
    /// collection of render targets. A target may be
    /// a source or a sink for a `Pass`
    pub targets: HashMap<String, Box<Target>>,
    /// Collection of scenes, having multiple scenes
    /// allows for selection of different fragments
    /// by different passes
    pub scenes: HashMap<String, Scene>,
    /// Collection of Cameras owned by the scene
    pub cameras: HashMap<String, Camera>,
}

impl Frame {
    /// Create an empty Frame
    pub fn new() -> Frame {
        Frame {
            layers: vec![],
            targets: HashMap::new(),
            scenes: HashMap::new(),
            cameras: HashMap::new(),
        }
    }
}
