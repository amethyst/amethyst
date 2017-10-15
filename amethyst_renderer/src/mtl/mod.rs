//! Physically-based material.

use gfx::traits::Pod;

use specs::{Component, DenseVecStorage};

use error::Result;
use tex::{Texture, TextureHandle, TextureBuilder};
use types::Factory;

/// Material struct.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Material {
    /// Diffuse map.
    pub albedo: TextureHandle,
    /// Emission map.
    pub emission: TextureHandle,
    /// Normal map.
    pub normal: TextureHandle,
    /// Metallic map.
    pub metallic: TextureHandle,
    /// Roughness map.
    pub roughness: TextureHandle,
    /// Ambient occlusion map.
    pub ambient_occlusion: TextureHandle,
    /// Caveat map.
    pub caveat: TextureHandle,
}

impl Component for Material {
    type Storage = DenseVecStorage<Self>;
}
