//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Sphere`s, Rectangle`s, and `Cube`s.

// TODO: Write a macro for defining the struct's.

/// An opaque handle for a "Mesh" resource.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MeshID {
    id: i64,
}

impl MeshID {
    /// Create a new instance.
    pub fn from_i64(value: i64) -> MeshID {
        MeshID { id: value }
    }
}

/// An opaque handle for a "Texture" resource.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TextureID {
    id: i64,
}

impl TextureID {
    /// Create a new instance.
    pub fn from_i64(value: i64) -> TextureID {
        TextureID { id: value }
    }
}

/// An opaque handle for a "Sphere" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SphereID {
    id: i64,
}

impl SphereID {
    pub fn from_i64(value: i64) -> SphereID {
        SphereID { id: value }
    }
}

/// An opaque handle for a "Cube" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CubeID {
    id: i64,
}

impl CubeID {
    pub fn from_i64(value: i64) -> CubeID {
        CubeID { id: value }
    }
}

/// An opaque handle for a "Recangle" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct RectangleID {
    id: i64,
}

impl RectangleID {
    pub fn from_i64(value: i64) -> RectangleID {
        RectangleID { id: value }
    }
}

pub trait PrefabGenerator {
    /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    fn gen_sphere(&mut self, m: MeshID, u: usize, v: usize) -> SphereID;
    fn gen_cube(&mut self, m: MeshID) -> CubeID;
    fn gen_rectangle(&mut self, m: MeshID, width: f32, height: f32) -> RectangleID;
}
