//! Physically-based material.

use {
    crate::types::Texture,
    amethyst_assets::Handle,
    amethyst_core::specs::prelude::{Component, DenseVecStorage},
};

/// Material reference this part of the texture
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TextureOffset {
    /// Start and end offset for U coordinate
    pub u: (f32, f32),
    /// Start and end offset for V coordinate
    pub v: (f32, f32),
}

impl Default for TextureOffset {
    fn default() -> Self {
        TextureOffset {
            u: (0., 1.),
            v: (0., 1.),
        }
    }
}

/// Material struct.
#[derive(Clone, PartialEq)]
pub struct Material {
    /// Alpha cutoff: the value at which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Diffuse map.
    pub albedo: Handle<Texture>,
    /// Diffuse texture offset
    pub albedo_offset: TextureOffset,
    /// Emission map.
    pub emission: Handle<Texture>,
    /// Emission texture offset
    pub emission_offset: TextureOffset,
    /// Normal map.
    pub normal: Handle<Texture>,
    /// Normal texture offset
    pub normal_offset: TextureOffset,
    /// Metallic map.
    pub metallic: Handle<Texture>,
    /// Metallic texture offset
    pub metallic_offset: TextureOffset,
    /// Roughness map.
    pub roughness: Handle<Texture>,
    /// Roughness texture offset
    pub roughness_offset: TextureOffset,
    /// Ambient occlusion map.
    pub ambient_occlusion: Handle<Texture>,
    /// Ambient occlusion texture offset
    pub ambient_occlusion_offset: TextureOffset,
    /// Caveat map.
    pub caveat: Handle<Texture>,
    /// Caveat texture offset
    pub caveat_offset: TextureOffset,
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
