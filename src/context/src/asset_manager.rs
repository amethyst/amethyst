//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Texture`s, `Mesh`es, and `Fragment`s.

extern crate amethyst_renderer;
extern crate gfx_device_gl;
extern crate gfx;
extern crate genmesh;
extern crate cgmath;

pub use self::gfx::tex::Kind;
use self::gfx::traits::FactoryExt;
use self::gfx::Factory;
use self::gfx::format::{Formatted, SurfaceTyped};
use self::amethyst_renderer::VertexPosNormal;
use self::amethyst_renderer::target::ColorFormat;

use self::genmesh::generators::{SphereUV, Cube};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};

use std::collections::HashMap;
use renderer::{Fragment, FragmentImpl};
use prefab_generator::{PrefabGenerator, CubeID, MeshID, RectangleID, SphereID, TextureID};

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

struct AssetIDManager {
    mesh: usize,
    texture: usize,
    sphere: usize,
    cube: usize,
    rectangle: usize,
}

impl AssetIDManager {
    pub fn new() -> AssetIDManager {
        AssetIDManager {
            mesh: 0,
            texture: 0,
            sphere: 0,
            cube: 0,
            rectangle: 0,
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

struct AssetIndex {
    meshes: HashMap<String, Mesh>,
    textures: HashMap<String, Texture>,

    // TODO: usize instead of String
    mmap: Vec<Option<Mesh>>,
    tmap: Vec<Option<Texture>>,
}

impl AssetIndex {
    fn new(capacity: usize) -> AssetIndex {
        AssetIndex {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            mmap: Vec::with_capacity(capacity),
            tmap: Vec::with_capacity(capacity),
        }
    }

    pub fn add_mesh(&mut self, id_manager: &mut AssetIDManager, m: Mesh) -> MeshID {
        let mesh_id = id_manager.next_mesh();
        self.mmap[mesh_id.to_vec_index()] = Some(m);
        mesh_id
    }

    pub fn add_texture(&mut self, id_manager: &mut AssetIDManager, t: Texture) -> TextureID {
        let texture_id = id_manager.next_texture();
        self.tmap[texture_id.to_vec_index()] = Some(t);
        texture_id
    }

    // pub fn get_mesh(&self, index: usize) -> &Option<Mesh> {
    // &self.mmap[index]
    // }

    // pub fn get_texture(&self, index: usize) -> &Option<Texture> {
    // &self.tmap[index]
    // }
}

/// A struct which allows loading and accessing assets.
pub struct AssetManager {
    id_manager: AssetIDManager,
    factory_impl: FactoryImpl,
    index: AssetIndex,
}

impl PrefabGenerator for AssetManager {
    fn gen_sphere(&mut self, u: usize, v: usize) -> SphereID {
        let data: Vec<VertexPosNormal> = SphereUV::new(u, v)
            .vertex(|(x, y, z)| {
                VertexPosNormal {
                    pos: [x, y, z],
                    normal: Vector3::new(x, y, z).normalize().into(),
                    tex_coord: [0., 0.],
                }
            })
            .triangulate()
            .vertices()
            .collect();
        let mesh = self.load_mesh(&data);
        let mesh_id = self.index.add_mesh(&mut self.id_manager, mesh);
        SphereID::from_meshid(mesh_id)
    }

    fn gen_cube(&mut self) -> CubeID {
        let data: Vec<VertexPosNormal> = Cube::new()
            .vertex(|(x, y, z)| {
                VertexPosNormal {
                    pos: [x, y, z],
                    normal: Vector3::new(x, y, z).normalize().into(),
                    tex_coord: [0., 0.],
                }
            })
            .triangulate()
            .vertices()
            .collect();
        let mesh = self.load_mesh(&data);
        CubeID::from_meshid(self.index.add_mesh(&mut self.id_manager, mesh))
    }

    fn gen_rectangle(&mut self, width: f32, height: f32) -> RectangleID {
        let data = vec![
            VertexPosNormal {
                pos: [-width/2., height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 1.],
            },
            VertexPosNormal {
                pos: [-width/2., -height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 0.],
            },
            VertexPosNormal {
                pos: [width/2., -height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [1., 0.],
            },
            VertexPosNormal {
                pos: [width/2., -height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 1.],
            },
            VertexPosNormal {
                pos: [width/2., height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [0., 0.],
            },
            VertexPosNormal {
                pos: [-width/2., height/2., 0.],
                normal: [0., 0., 1.],
                tex_coord: [1., 0.],
            },
        ];
        let mesh = self.load_mesh(&data);
        RectangleID::from_meshid(self.index.add_mesh(&mut self.id_manager, mesh))
    }
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
            id_manager: AssetIDManager::new(),
            factory_impl: factory_impl,
            index: AssetIndex::new(capacity),
        }
    }
    /// Load a `Mesh` from vertex data.
    fn load_mesh(&mut self, data: &Vec<VertexPosNormal>) -> Mesh {
        match self.factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let (buffer, slice) = factory.create_vertex_buffer_with_slice(&data, ());
                let mesh_impl = MeshImpl::OpenGL {
                    buffer: buffer,
                    slice: slice,
                };
                Mesh { mesh_impl: mesh_impl };
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null {} => unimplemented!(),
        }
        unimplemented!()
    }
    /// Lookup a `Mesh` by name.
    pub fn get_mesh(&mut self, name: &str) -> Option<Mesh> {
        match self.index.meshes.get(name.into()) {
            Some(mesh) => Some((*mesh).clone()),
            None => None,
        }
    }
    /// Load a `Texture` from pixel data.
    pub fn load_texture(&mut self, name: &str, kind: Kind, data: &[&[<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]]) {
        match self.factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let shader_resource_view = match factory.create_texture_const::<ColorFormat>(kind, data) {
                    Ok((_, shader_resource_view)) => shader_resource_view,
                    Err(_) => return,
                };
                let texture = amethyst_renderer::Texture::Texture(shader_resource_view);
                let texture_impl = TextureImpl::OpenGL { texture: texture };
                let texture = Texture { texture_impl: texture_impl };
                self.index.textures.insert(name.into(), texture);
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null => (),
        }
    }
    /// Create a constant solid color `Texture` from a specified color.
    pub fn create_constant_texture(&mut self, name: &str, color: [f32; 4]) {
        let texture = amethyst_renderer::Texture::Constant(color);
        let texture_impl = TextureImpl::OpenGL { texture: texture };
        let texture = Texture { texture_impl: texture_impl };
        self.index.textures.insert(name.into(), texture);
    }
    /// Lookup a `Texture` by name.
    pub fn get_texture(&mut self, name: &str) -> Option<Texture> {
        match self.index.textures.get(name.into()) {
            Some(texture) => Some((*texture).clone()),
            None => None,
        }
    }
    /// Construct and return a `Fragment` from previously loaded mesh, ka and kd textures and a transform matrix.
    pub fn get_fragment(&mut self, mesh: &str, ka: &str, kd: &str, transform: [[f32; 4]; 4]) -> Option<Fragment> {
        let mesh = match self.get_mesh(mesh) {
            Some(mesh) => mesh,
            None => return None,
        };
        let ka = match self.get_texture(ka) {
            Some(ka) => ka,
            None => return None,
        };
        let kd = match self.get_texture(kd) {
            Some(kd) => kd,
            None => return None,
        };
        match self.factory_impl {
            FactoryImpl::OpenGL { .. } => {
                let ka = match ka.texture_impl {
                    TextureImpl::OpenGL { texture } => texture,
                    #[cfg(windows)]
                    TextureImpl::Direct3D {} => return None,
                    TextureImpl::Null => return None,
                };

                let kd = match kd.texture_impl {
                    TextureImpl::OpenGL { texture } => texture,
                    #[cfg(windows)]
                    TextureImpl::Direct3D {} => return None,
                    TextureImpl::Null => return None,
                };

                let (buffer, slice) = match mesh.mesh_impl {
                    MeshImpl::OpenGL { buffer, slice } => (buffer, slice),
                    #[cfg(windows)]
                    MeshImpl::Direct3D {} => return None,
                    MeshImpl::Null => return None,
                };

                let fragment = amethyst_renderer::Fragment {
                    transform: transform,
                    buffer: buffer,
                    slice: slice,
                    ka: ka,
                    kd: kd,
                };
                let fragment_impl = FragmentImpl::OpenGL { fragment: fragment };
                Some(Fragment { fragment_impl: fragment_impl })
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null => None,
        }
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
