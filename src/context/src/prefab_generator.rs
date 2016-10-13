//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Sphere`s, Rectangle`s, and `Cube`s.

extern crate amethyst_renderer;
extern crate genmesh;
extern crate cgmath;

use self::amethyst_renderer::VertexPosNormal;
use self::genmesh::generators::{SphereUV, Cube};
use self::genmesh::{MapToVertices, Triangulate, Vertices};
use self::cgmath::{Vector3, InnerSpace};
use asset_manager::{AssetManager, Mesh, MeshID};

// TODO: Write a macro for defining the struct's.

/// An opaque handle for a "Sphere" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
#[must_use]
pub struct SphereID {
    id: MeshID,
}

impl SphereID {
    /// Create a new instance.
    pub fn from_meshid(value: MeshID) -> SphereID {
        SphereID { id: value }
    }

    /// Convertible to the underlying `MeshID`.
    pub fn to_meshid<'a>(&'a self) -> &'a MeshID {
        &self.id
    }
}

/// An opaque handle for a "Cube" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
#[must_use]
pub struct CubeID {
    id: MeshID,
}

impl CubeID {
    /// Create a new instance.
    pub fn from_meshid(value: MeshID) -> CubeID {
        CubeID { id: value }
    }

    /// Convertible to the underlying `MeshID`.
    pub fn to_meshid<'a>(&'a self) -> &'a MeshID {
        &self.id
    }
}

/// An opaque handle for a "Recangle" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
#[must_use]
pub struct RectangleID {
    id: MeshID,
}

impl RectangleID {
    /// Create a new instance.
    pub fn from_meshid(value: MeshID) -> RectangleID {
        RectangleID { id: value }
    }

    /// Convertible to the underlying `MeshID`.
    pub fn to_meshid<'a>(&'a self) -> &'a MeshID {
        &self.id
    }
}

pub trait PrefabGenerator {
    /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    fn gen_sphere(&mut self, u: usize, v: usize) -> SphereID;
    fn gen_cube(&mut self) -> CubeID;
    fn gen_rectangle(&mut self, width: f32, height: f32) -> RectangleID;
}

pub trait PrefabIndex {
    /// Load `Prefab`s`
    fn load_sphere(&self, id: &SphereID) -> &Option<Mesh>;
    fn load_cube(&self, id: &CubeID) -> &Option<Mesh>;
    fn load_rectangle(&self, id: &RectangleID) -> &Option<Mesh>;
}

/// A struct which allows loading and accessing `prefabs`.
pub struct PrefabManager {
    assets: AssetManager,
}

impl PrefabIndex for PrefabManager {
    fn load_sphere(&self, id: &SphereID) -> &Option<Mesh> {
        self.assets.get_mesh(id.to_meshid())
    }
    fn load_cube(&self, id: &CubeID) -> &Option<Mesh> {
        self.assets.get_mesh(id.to_meshid())
    }
    fn load_rectangle(&self, id: &RectangleID) -> &Option<Mesh> {
        self.assets.get_mesh(id.to_meshid())
    }
}

impl PrefabGenerator for PrefabManager {
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
        let mesh_id = self.assets.load_mesh(&data);
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
        let mesh_id = self.assets.load_mesh(&data);
        CubeID::from_meshid(mesh_id)
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
        let mesh_id = self.assets.load_mesh(&data);
        RectangleID::from_meshid(mesh_id)
    }
}
