//! This module provides a `resource manager`.
//! which loads and provides access to resources`.

extern crate amethyst_renderer;
extern crate gfx_device_gl;
extern crate gfx;
extern crate cgmath;

pub use self::gfx::tex::Kind;
use self::gfx::traits::FactoryExt;
use self::gfx::Factory;
use self::gfx::format::{Formatted, SurfaceTyped};
use self::amethyst_renderer::VertexPosNormal;
use self::amethyst_renderer::target::ColorFormat;

use renderer::{Fragment, FragmentImpl};
use resource::{MeshID, ResourceIndex, ResourceIDFactory, TextureID};

/// An enum with variants representing concrete
/// `Factory` types compatible with different backends.
pub enum FactoryImpl {
    OpenGL { factory: gfx_device_gl::Factory },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

#[derive(Debug)]
pub enum DeviceError {
    TextureLoading(&'static str),
}

/// A wraper around `Buffer` and `Slice` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Mesh {
    pub mesh_impl: MeshImpl,
}

/// An enum with variants representing concrete
/// `Mesh` types compatible with different backends.
#[derive(Clone)]
pub enum MeshImpl {
    OpenGL {
        buffer: gfx::handle::Buffer<gfx_device_gl::Resources, VertexPosNormal>,
        slice: gfx::Slice<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

/// A wraper around `Texture` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Texture {
    pub texture_impl: TextureImpl,
}

/// An enum with variants representing concrete
/// `Texture` types compatible with different backends.
#[derive(Clone)]
pub enum TextureImpl {
    OpenGL { texture: amethyst_renderer::Texture<gfx_device_gl::Resources>, },
    #[cfg(windows)]
    Direct3D {
        // stub
    },
    Null,
}

pub trait DeviceManager {
    fn load_mesh(&mut self, data: &Vec<VertexPosNormal>) -> MeshID;
    fn get_mesh(&self, id: &MeshID) -> &Option<Mesh>;
    fn load_texture(&mut self, kind: Kind, data: &[&[<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]]) -> Result<TextureID, DeviceError>;
    fn create_constant_texture(&mut self, color: [f32; 4]) -> TextureID;
    fn get_texture(&mut self, id: &TextureID) -> &Option<Texture>;
    fn make_fragment(&mut self, mesh: Mesh, ka: Texture, kd: Texture, transform: [[f32; 4]; 4]) -> Fragment;
}

/// A struct which allows loading and accessing resources.
pub struct DeviceManagerImpl {
    factory_impl: FactoryImpl,
    index: ResourceIndex,
    id_factory: ResourceIDFactory,
}

impl DeviceManagerImpl {
    /// Create a new `DeviceManager` from `FactoryImpl` (used internally).
    ///
    /// Assumes a default capacity for the underlying storage.
    pub fn new(factory_impl: FactoryImpl) -> DeviceManagerImpl {
        DeviceManagerImpl::new_sized(factory_impl, 10000)
    }
    /// Create a new `DeviceManager` from `FactoryImpl` (used internally).
    /// Allows a capacity parameter to be passed, reserving capacity internally (optimization
    /// opportunity for the user).
    pub fn new_sized(factory_impl: FactoryImpl, capacity: usize) -> DeviceManagerImpl {
        DeviceManagerImpl {
            factory_impl: factory_impl,
            index: ResourceIndex::new(capacity),
            id_factory: ResourceIDFactory::new(),
        }
    }
}

impl DeviceManager for DeviceManagerImpl {
    fn load_mesh(&mut self, data: &Vec<VertexPosNormal>) -> MeshID {
        match self.factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let (buffer, slice) = factory.create_vertex_buffer_with_slice(&data, ());
                let mesh_impl = MeshImpl::OpenGL {
                    buffer: buffer,
                    slice: slice,
                };
                let mesh = Mesh { mesh_impl: mesh_impl };
                let mesh_id = self.index.add_mesh(&mut self.id_factory, mesh);
                return mesh_id;
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {}
            FactoryImpl::Null {} => {}
        }
        unimplemented!()
    }
    /// Lookup a `Mesh` by name.
    fn get_mesh(&self, id: &MeshID) -> &Option<Mesh> {
        self.index.get_mesh(id)
    }
    /// Load a `Texture` from pixel data.
    fn load_texture(&mut self, kind: Kind, data: &[&[<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]]) -> Result<TextureID, DeviceError> {
        match self.factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let shader_resource_view = match factory.create_texture_const::<ColorFormat>(kind, data) {
                    Ok((_, shader_resource_view)) => shader_resource_view,
                    Err(_) => return Err(DeviceError::TextureLoading("TODO")),
                };
                let texture = amethyst_renderer::Texture::Texture(shader_resource_view);
                let texture_impl = TextureImpl::OpenGL { texture: texture };
                let texture = Texture { texture_impl: texture_impl };
                let texture_id = self.index.add_texture(&mut self.id_factory, texture);
                return Ok(texture_id);
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {}
            FactoryImpl::Null => (),
        }
        unimplemented!();
    }
    /// Create a constant solid color `Texture` from a specified color.
    fn create_constant_texture(&mut self, color: [f32; 4]) -> TextureID {
        let texture = amethyst_renderer::Texture::Constant(color);
        let texture_impl = TextureImpl::OpenGL { texture: texture };
        let texture = Texture { texture_impl: texture_impl };
        self.index.add_texture(&mut self.id_factory, texture)
    }
    /// Lookup a `Texture` by name.
    fn get_texture(&mut self, id: &TextureID) -> &Option<Texture> {
        self.index.get_texture(id)
    }
    fn make_fragment(&mut self, mesh: Mesh, ka: Texture, kd: Texture, transform: [[f32; 4]; 4]) -> Fragment {
        match self.factory_impl {
            FactoryImpl::OpenGL { .. } => {
                let ka = match ka.texture_impl {
                    TextureImpl::OpenGL { texture } => texture,
                    #[cfg(windows)]
                    TextureImpl::Direct3D {} => unimplemented!(),
                    TextureImpl::Null => unimplemented!(),
                };

                let kd = match kd.texture_impl {
                    TextureImpl::OpenGL { texture } => texture,
                    #[cfg(windows)]
                    TextureImpl::Direct3D {} => unimplemented!(),
                    TextureImpl::Null => unimplemented!(),
                };

                let (buffer, slice) = match mesh.mesh_impl {
                    MeshImpl::OpenGL { buffer, slice } => (buffer, slice),
                    #[cfg(windows)]
                    MeshImpl::Direct3D {} => unimplemented!(),
                    MeshImpl::Null => unimplemented!(),
                };

                let fragment = amethyst_renderer::Fragment {
                    transform: transform,
                    buffer: buffer,
                    slice: slice,
                    ka: ka,
                    kd: kd,
                };
                let fragment_impl = FragmentImpl::OpenGL { fragment: fragment };
                Fragment { fragment_impl: fragment_impl }
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null => unimplemented!(),
        }
    }
}
