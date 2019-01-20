//! Physically-based material.

use amethyst_core::specs::prelude::{Component, DenseVecStorage};

use crate::tex::TextureView;

/// Material struct.
#[derive(Clone, PartialEq)]
pub struct Material {
    /// Alpha cutoff: the value at which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Diffuse map.
    pub albedo: TextureView,
    /// Emission map.
    pub emission: TextureView,
    /// Normal map.
    pub normal: TextureView,
    /// Metallic map.
    pub metallic: TextureView,
    /// Roughness map.
    pub roughness: TextureView,
    /// Ambient occlusion map.
    pub ambient_occlusion: TextureView,
    /// Caveat map.
    pub caveat: TextureView,
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
