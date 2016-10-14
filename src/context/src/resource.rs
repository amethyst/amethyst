//! This module defines different `resources` as well as tools for managing them.
use device::{Mesh, Texture};

/// An opaque handle for a "Mesh" resource.
#[must_use]
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
#[must_use]
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

pub struct ResourceIDFactory {
    mesh: usize,
    texture: usize,
}

impl ResourceIDFactory {
    pub fn new() -> ResourceIDFactory {
        ResourceIDFactory {
            mesh: 0,
            texture: 0,
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
}

pub struct ResourceIndex {
    mmap: Vec<Option<Mesh>>,
    tmap: Vec<Option<Texture>>,
}

impl ResourceIndex {
    pub fn new(capacity: usize) -> ResourceIndex {
        ResourceIndex {
            mmap: Vec::with_capacity(capacity),
            tmap: Vec::with_capacity(capacity),
        }
    }

    pub fn add_mesh(&mut self, id_factory: &mut ResourceIDFactory, m: Mesh) -> MeshID {
        let mesh_id = id_factory.next_mesh();
        self.mmap[mesh_id.to_vec_index()] = Some(m);
        mesh_id
    }

    pub fn add_texture(&mut self, id_factory: &mut ResourceIDFactory, t: Texture) -> TextureID {
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
