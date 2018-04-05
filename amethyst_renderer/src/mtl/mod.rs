//! Physically-based material.

use amethyst_core::specs::{Component, DenseVecStorage};

use tex::TextureHandle;

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

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Clone)]
pub struct MaterialDefaults(pub Material);
