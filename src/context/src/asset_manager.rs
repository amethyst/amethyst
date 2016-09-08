//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Texture`s, `Mesh`es, and `Fragment`s.

extern crate amethyst_renderer;
extern crate gfx_device_gl;
extern crate gfx;
extern crate genmesh;
extern crate cgmath;

pub use self::gfx::tex::Kind;
use self::gfx::Factory;
use self::gfx::traits::FactoryExt;
use self::gfx::format::{Formatted, SurfaceTyped};
use self::amethyst_renderer::VertexPosNormal;
use self::amethyst_renderer::target::ColorFormat;

use self::genmesh::generators::{SphereUV, Cube};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, EuclideanVector};

use std::collections::HashMap;
use renderer::{Fragment};

/// A struct which allows loading and accessing assets.
pub struct AssetManager {
    factory: gfx_device_gl::Factory,
    meshes: HashMap<String, Mesh>,
    textures: HashMap<String, Texture>,
}

impl AssetManager {
    /// Create a new `AssetManager` from `Factory` (used internally).
    pub fn new(factory: gfx_device_gl::Factory) -> AssetManager {
        AssetManager {
            factory: factory,
            meshes: HashMap::new(),
            textures: HashMap::new(),
        }
    }
    /// Load a `Mesh` from vertex data.
    pub fn load_mesh(&mut self, name: &str, data: &Vec<VertexPosNormal>) {
        let (buffer, slice) = self.factory.create_vertex_buffer_with_slice(&data, ());
        let mesh = Mesh {
            buffer: buffer,
            slice: slice
        };
        self.meshes.insert(name.into(), mesh);
    }
    /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    pub fn gen_sphere(&mut self, name: &str, u: usize, v: usize) {
        let data: Vec<VertexPosNormal> =
            SphereUV::new(u, v)
            .vertex(|(x, y, z)| VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.]
            })
            .triangulate()
            .vertices()
            .collect();
        self.load_mesh(name, &data);
    }
    /// Generate and load a cube mesh.
    pub fn gen_cube(&mut self, name: &str) {
        let data: Vec<VertexPosNormal> =
            Cube::new()
            .vertex(|(x, y, z)| VertexPosNormal {
                pos: [x, y, z],
                normal: Vector3::new(x, y, z).normalize().into(),
                tex_coord: [0., 0.]
            })
            .triangulate()
            .vertices()
            .collect();
        self.load_mesh(name, &data);
    }
    /// Lookup a `Mesh` by name.
    pub fn get_mesh(&mut self, name: &str) -> Option<Mesh> {
        match self.meshes.get(name.into()) {
            Some(mesh) => {
                Some((*mesh).clone())
            },
            None => None,
        }
    }
    /// Load a `Texture` from pixel data.
    pub fn load_texture(&mut self, name: &str, kind: Kind, data: &[&[<<ColorFormat as Formatted>::Surface as SurfaceTyped>::DataType]]) {
        let shader_resource_view = match self.factory.create_texture_const::<ColorFormat>(kind, data) {
            Ok((_, shader_resource_view)) => shader_resource_view,
            Err(_) => return,
        };
        let texture = amethyst_renderer::Texture::Texture(shader_resource_view);
        let texture = Texture {
            texture: texture,
        };
        self.textures.insert(name.into(), texture);
    }
    /// Create a constant solid color `Texture` from a specified color.
    pub fn create_constant_texture(&mut self, name: &str, color: [f32; 4]) {
        let texture = amethyst_renderer::Texture::Constant(color);
        let texture = Texture {
            texture: texture,
        };
        self.textures.insert(name.into(), texture);
    }
    /// Lookup a `Texture` by name.
    pub fn get_texture(&mut self, name: &str) -> Option<Texture> {
        match self.textures.get(name.into()) {
            Some(texture) => {
                Some((*texture).clone())
            },
            None => None,
        }
    }
    /// Construct and return a `Fragment` from previously loaded mesh, ka and kd textures and a transform matrix.
    pub fn get_fragment(&mut self, mesh: &str, ka: &str, kd: &str, transform: [[f32; 4]; 4]) -> Option<Fragment> {
        // TODO: These unwrap calls shouldn't be here, this function should return None in that
        // case. I couldn't figure out an equivalent of try!() for option though, maybe someone on
        // the code review will suggest someway to improve this idiomatically.
        let mesh = self.get_mesh(mesh).unwrap();
        let ka = self.get_texture(ka).unwrap();
        let kd = self.get_texture(kd).unwrap();

        let fragment = amethyst_renderer::Fragment {
            transform: transform,
            buffer: mesh.buffer,
            slice: mesh.slice,
            ka: ka.texture,
            kd: kd.texture,
        };
        Some(Fragment {
            data: fragment,
        })
    }
}

/// A wraper around `Buffer` and `Slice` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Mesh {
    buffer: gfx::handle::Buffer<gfx_device_gl::Resources, VertexPosNormal>,
    slice: gfx::Slice<gfx_device_gl::Resources>,
}

/// A wraper around `Texture` required to
/// hide all platform specific code from the user.
#[derive(Clone)]
pub struct Texture {
    texture: amethyst_renderer::Texture<gfx_device_gl::Resources>,
}
