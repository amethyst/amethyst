//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Texture`s, `Mesh`es, and `Fragment`s.

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

macro_rules! try_refopt_clone {
    ($expr:expr) => (match *$expr {
        ::std::option::Option::Some(ref val) => val.clone(),
        ::std::option::Option::None => return None
    })
}

/// An opaque handle for a "Mesh" resource.
#[derive(Clone, Eq, PartialEq, Hash)]
#[must_use]
pub struct MeshID {
    id: usize,
}

impl MeshID {
    /// Create a new instance.
    pub fn from_usize(value: usize) -> MeshID {
        MeshID { id: value }
    }

    /// Convertible to the underlying `usize`, so they can be used as an index to a Vec
    /// conveniently.
    pub fn to_vec_index(&self) -> usize {
        self.id
    }
}

/// An opaque handle for a "Texture" resource.
#[derive(Clone, Eq, PartialEq, Hash)]
#[must_use]
pub struct TextureID {
    id: usize,
}

impl TextureID {
    /// Create a new instance.
    pub fn from_usize(value: usize) -> TextureID {
        TextureID { id: value }
    }

    /// Convertible to the underlying `usize`, so they can be used as an index to a Vec
    /// conveniently.
    pub fn to_vec_index(&self) -> usize {
        self.id
    }
}

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

pub struct AssetIDFactory {
    mesh: usize,
    texture: usize, /* sphere: usize,
                     * cube: usize,
                     * rectangle: usize, */
}

impl AssetIDFactory {
    pub fn new() -> AssetIDFactory {
        AssetIDFactory {
            mesh: 0,
            texture: 0, /* sphere: 0,
                         * cube: 0,
                         * rectangle: 0, */
        }
    }

    pub fn next_mesh(&mut self) -> MeshID {
        let id = MeshID::from_usize(self.mesh);
        self.mesh += 1;
        id
    }

    pub fn next_texture(&mut self) -> TextureID {
        let id = TextureID::from_usize(self.texture);
        self.texture += 1;
        id
    }

    // pub fn next_sphere(&mut self) -> SphereID {
    // let id = SphereID::from_usize(self.sphere);
    // self.sphere += 1;
    // id
    // }
    //
    // pub fn next_cube(&mut self) -> CubeID {
    // let id = CubeID::from_usize(self.cube);
    // self.cube += 1;
    // id
    // }
    //
    // pub fn next_rectangle(&mut self) -> RectangleID {
    // let id = RectangleID::from_usize(self.rectangle);
    // self.rectangle += 1;
    // id
    // }
    //
}

pub struct AssetIndex {
    mmap: Vec<Option<Mesh>>,
    tmap: Vec<Option<Texture>>,
}

impl AssetIndex {
    fn new(capacity: usize) -> AssetIndex {
        AssetIndex {
            mmap: Vec::with_capacity(capacity),
            tmap: Vec::with_capacity(capacity),
        }
    }

    pub fn add_mesh(&mut self, id_factory: &mut AssetIDFactory, m: Mesh) -> MeshID {
        let mesh_id = id_factory.next_mesh();
        self.mmap[mesh_id.to_vec_index()] = Some(m);
        mesh_id
    }

    pub fn add_texture(&mut self, id_factory: &mut AssetIDFactory, t: Texture) -> TextureID {
        let texture_id = id_factory.next_texture();
        self.tmap[texture_id.to_vec_index()] = Some(t);
        texture_id
    }

    pub fn get_mesh(&self, index: &MeshID) -> &Option<Mesh> {
        &self.mmap[index.to_vec_index()]
    }

    pub fn get_texture(&self, index: &TextureID) -> &Option<Texture> {
        &self.tmap[index.to_vec_index()]
    }
}

#[derive(Debug)]
pub enum ResourceError {
    TextureLoading(&'static str),
}

/// A struct which allows loading and accessing assets.
pub struct AssetManager {
    factory_impl: FactoryImpl,
    index: AssetIndex,
    id_factory: AssetIDFactory,
}

impl AssetManager {
    /// Create a new `AssetManager` from `FactoryImpl` (used internally).
    ///
    /// Assumes a default capacity for the underlying storage.
    pub fn new(factory_impl: FactoryImpl) -> AssetManager {
        AssetManager::new_sized(factory_impl, 10000)
    }
    /// Create a new `AssetManager` from `FactoryImpl` (used internally).
    /// Allows a capacity parameter to be passed, reserving capacity internally (optimization
    /// opportunity).
    pub fn new_sized(factory_impl: FactoryImpl, capacity: usize) -> AssetManager {
        AssetManager {
            factory_impl: factory_impl,
            index: AssetIndex::new(capacity),
            id_factory: AssetIDFactory::new(),
        }
    }
    pub fn load_mesh(&mut self, data: &Vec<VertexPosNormal>) -> MeshID {
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
    pub fn get_mesh(&self, id: &MeshID) -> &Option<Mesh> {
        self.index.get_mesh(id)
    }
    /// Load a `Texture` from pixel data.
    pub fn load_texture(&mut self, kind: Kind, data: &[&[<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]]) -> Result<TextureID, ResourceError> {
        match self.factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let shader_resource_view = match factory.create_texture_const::<ColorFormat>(kind, data) {
                    Ok((_, shader_resource_view)) => shader_resource_view,
                    Err(_) => return Err(ResourceError::TextureLoading("hi ben")),
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
    pub fn create_constant_texture(&mut self, color: [f32; 4]) -> TextureID {
        let texture = amethyst_renderer::Texture::Constant(color);
        let texture_impl = TextureImpl::OpenGL { texture: texture };
        let texture = Texture { texture_impl: texture_impl };
        self.index.add_texture(&mut self.id_factory, texture)
    }
    /// Lookup a `Texture` by name.
    pub fn get_texture(&mut self, id: &TextureID) -> &Option<Texture> {
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
    /// Construct and return a `Fragment` from previously loaded mesh, ka and kd textures and a transform matrix.
    pub fn get_fragment(&mut self, m: &MeshID, ka: &TextureID, kd: &TextureID, transform: [[f32; 4]; 4]) -> Option<Fragment> {
        // We clone the components of the Fragment, only if it is present.
        let mesh = try_refopt_clone!(self.get_mesh(m));
        let ka = try_refopt_clone!(self.get_texture(ka));
        let kd = try_refopt_clone!(self.get_texture(kd));
        Some(self.make_fragment(mesh, ka, kd, transform))
    }
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

/// A wraper around `Buffer` and `Slice` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Mesh {
    mesh_impl: MeshImpl,
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

/// A wraper around `Texture` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Texture {
    texture_impl: TextureImpl,
}
