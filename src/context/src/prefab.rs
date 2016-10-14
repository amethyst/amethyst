//! This module provides an asset manager
//! which loads and provides access to device_manager,
//! such as `Sphere`s, Rectangle`s, and `Cube`s.

extern crate amethyst_renderer;
extern crate genmesh;
extern crate cgmath;

use self::amethyst_renderer::VertexPosNormal;
use self::genmesh::generators::{SphereUV, Cube};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};
use device::{Mesh, DeviceManager, DeviceManagerImpl};
use resource::{MeshID, TextureID};
use renderer::Fragment;

macro_rules! try_refopt_clone {
    ($expr:expr) => (match *$expr {
        ::std::option::Option::Some(ref val) => val.clone(),
        ::std::option::Option::None => return None
    })
}

// TODO: Write more macros for defining the struct's.

pub trait PrefabGenerator {
    /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    fn gen_sphere(&mut self, u: usize, v: usize) -> MeshID;
    fn gen_cube(&mut self) -> MeshID;
    fn gen_rectangle(&mut self, width: f32, height: f32) -> MeshID;
}

pub trait PrefabIndex {
    /// Load `Prefab`s`
    fn load_sphere(&self, id: &MeshID) -> &Option<Mesh>;
    fn load_cube(&self, id: &MeshID) -> &Option<Mesh>;
    fn load_rectangle(&self, id: &MeshID) -> &Option<Mesh>;

    /// Construct and return a `Fragment` from previously loaded mesh, ka and kd textures and a transform matrix.
    fn get_fragment(&mut self, m: &MeshID, ka: &TextureID, kd: &TextureID, transform: [[f32; 4]; 4]) -> Option<Fragment>;
}

/// A struct which allows loading and accessing `prefabs`.
pub struct PrefabManager {
    device_manager: DeviceManagerImpl,
}

impl PrefabManager {
    pub fn new(r: DeviceManagerImpl) -> PrefabManager {
        PrefabManager { device_manager: r }
    }

    /// Create a constant solid color `Texture` from a specified color.
    pub fn create_constant_texture(&mut self, color: [f32; 4]) -> TextureID {
        self.device_manager.create_constant_texture(color)
    }
}

impl PrefabIndex for PrefabManager {
    fn load_sphere(&self, id: &MeshID) -> &Option<Mesh> {
        self.device_manager.get_mesh(id)
    }
    fn load_cube(&self, id: &MeshID) -> &Option<Mesh> {
        self.device_manager.get_mesh(id)
    }
    fn load_rectangle(&self, id: &MeshID) -> &Option<Mesh> {
        self.device_manager.get_mesh(id)
    }
    fn get_fragment(&mut self, m: &MeshID, ka: &TextureID, kd: &TextureID, transform: [[f32; 4]; 4]) -> Option<Fragment> {
        // We clone the components of the Fragment, only if it is present.
        let mesh = try_refopt_clone!(self.device_manager.get_mesh(m));
        let ka = try_refopt_clone!(self.device_manager.get_texture(ka));
        let kd = try_refopt_clone!(self.device_manager.get_texture(kd));
        Some(self.device_manager.make_fragment(mesh, ka, kd, transform))
    }
}

impl PrefabGenerator for PrefabManager {
    fn gen_sphere(&mut self, u: usize, v: usize) -> MeshID {
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
        self.device_manager.load_mesh(&data)
    }

    fn gen_cube(&mut self) -> MeshID {
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
        self.device_manager.load_mesh(&data)
    }

    fn gen_rectangle(&mut self, width: f32, height: f32) -> MeshID {
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
        self.device_manager.load_mesh(&data)
    }
}
