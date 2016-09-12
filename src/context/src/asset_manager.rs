//! This module provides an asset manager which loads and provides access to assets,
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

/// An enum with variants representing concrete `Factory` types compatible with
/// different backends.
pub enum FactoryImpl {
    OpenGL { factory: gfx_device_gl::Factory },
    #[cfg(windows)]
    Direct3D {
        // TODO: Implementation needed.
    },
    Null,
}

/// Manages the loading and accessing of game assets.
pub struct AssetManager {
    factory_impl: FactoryImpl,
    meshes: HashMap<String, Mesh>,
    textures: HashMap<String, Texture>,
}

impl AssetManager {
    /// Creates a new `AssetManager` from the given `FactoryImpl` (used internally).
    pub fn new(factory_impl: FactoryImpl) -> AssetManager {
        AssetManager {
            factory_impl: factory_impl,
            meshes: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    /// Loads a `Mesh` from vertex data.
    pub fn load_mesh(&mut self, name: &str, data: &Vec<VertexPosNormal>) {
        match self.factory_impl {
            FactoryImpl::OpenGL { ref mut factory } => {
                let (buffer, slice) = factory.create_vertex_buffer_with_slice(&data, ());
                let mesh_impl = MeshImpl::OpenGL {
                    buffer: buffer,
                    slice: slice,
                };
                let mesh = Mesh { mesh_impl: mesh_impl };
                self.meshes.insert(name.into(), mesh);
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null => (),
        }
    }

    /// Generates and loads a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    pub fn gen_sphere(&mut self, name: &str, u: usize, v: usize) {
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
        self.load_mesh(name, &data);
    }

    /// Generates and loads a cube mesh.
    pub fn gen_cube(&mut self, name: &str) {
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
        self.load_mesh(name, &data);
    }

    /// Generates and loads a rectangle mesh in the XY plane with given `width`
    /// and `height`.
    pub fn gen_rectangle(&mut self, name: &str, width: f32, height: f32) {
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
        self.load_mesh(name, &data);
    }

    /// Looks up a `Mesh` asset by name.
    pub fn get_mesh(&mut self, name: &str) -> Option<Mesh> {
        match self.meshes.get(name.into()) {
            Some(mesh) => Some((*mesh).clone()),
            None => None,
        }
    }

    /// Loads a `Texture` from pixel data.
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
                self.textures.insert(name.into(), texture);
            }
            #[cfg(windows)]
            FactoryImpl::Direct3D {} => {
                unimplemented!();
            }
            FactoryImpl::Null => (),
        }
    }

    /// Creates a new `Texture` asset with the specified color.
    pub fn create_constant_texture(&mut self, name: &str, color: [f32; 4]) {
        let texture = amethyst_renderer::Texture::Constant(color);
        let texture_impl = TextureImpl::OpenGL { texture: texture };
        let texture = Texture { texture_impl: texture_impl };
        self.textures.insert(name.into(), texture);
    }

    /// Looks up a `Texture` asset by name.
    pub fn get_texture(&mut self, name: &str) -> Option<Texture> {
        match self.textures.get(name.into()) {
            Some(texture) => Some((*texture).clone()),
            None => None,
        }
    }

    /// Constructs and returns a `Fragment` from previously loaded mesh, *ka*
    /// and *kd* textures, and a transform matrix.
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

/// An enum with variants representing concrete `Mesh` types compatible with
/// different backends.
#[derive(Clone)]
pub enum MeshImpl {
    OpenGL {
        buffer: gfx::handle::Buffer<gfx_device_gl::Resources, VertexPosNormal>,
        slice: gfx::Slice<gfx_device_gl::Resources>,
    },
    #[cfg(windows)]
    Direct3D {
        // TODO: Implementation needed.
    },
    Null,
}

/// A backend-agnostic wrapper around `Buffer` and `Slice`.
#[derive(Clone)]
pub struct Mesh {
    mesh_impl: MeshImpl,
}

/// An enum with variants representing concrete `Texture` types compatible with
/// different backends.
#[derive(Clone)]
pub enum TextureImpl {
    OpenGL { texture: amethyst_renderer::Texture<gfx_device_gl::Resources>, },
    #[cfg(windows)]
    Direct3D {
        // TODO: Implementation needed.
    },
    Null,
}

/// A backend-agnostic wrapper around `Texture`.
#[derive(Clone)]
pub struct Texture {
    texture_impl: TextureImpl,
}
