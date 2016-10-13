//! This module provides an asset manager
//! which loads and provides access to assets,
//! such as `Sphere`s, Rectangle`s, and `Cube`s.

// TODO: Write a macro for defining the struct's.

/// An opaque handle for a "Mesh" resource.
#[derive(Clone, Eq, PartialEq, Hash)]
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

/// An opaque handle for a "Sphere" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SphereID {
    id: usize,
}

impl SphereID {
    /// Create a new instance.
    pub fn from_meshid(value: MeshID) -> SphereID {
        SphereID { id: value.to_vec_index() }
    }

    /// Convertible to the underlying `usize`, so they can be used as an index to a Vec
    /// conveniently.
    pub fn to_vec_index(&self) -> usize {
        self.id
    }
}

/// An opaque handle for a "Cube" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CubeID {
    id: usize,
}

impl CubeID {
    /// Create a new instance.
    pub fn from_meshid(value: MeshID) -> CubeID {
        CubeID { id: value.to_vec_index() }
    }

    /// Convertible to the underlying `usize`, so they can be used as an index to a Vec
    /// conveniently.
    pub fn to_vec_index(&self) -> usize {
        self.id
    }
}

/// An opaque handle for a "Recangle" prefab.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct RectangleID {
    id: usize,
}

impl RectangleID {
    /// Create a new instance.
    pub fn from_meshid(value: MeshID) -> RectangleID {
        RectangleID { id: value.to_vec_index() }
    }

    /// Convertible to the underlying `usize`, so they can be used as an index to a Vec
    /// conveniently.
    pub fn to_vec_index(&self) -> usize {
        self.id
    }
}

pub trait PrefabGenerator {
    /// Generate and load a sphere mesh using the number of vertices accross the equator (u)
    /// and the number of vertices from pole to pole (v).
    fn gen_sphere(&mut self, u: usize, v: usize) -> SphereID;
    fn gen_cube(&mut self) -> CubeID;
    fn gen_rectangle(&mut self, width: f32, height: f32) -> RectangleID;
}
