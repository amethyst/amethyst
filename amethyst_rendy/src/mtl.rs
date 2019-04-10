//! Physically-based material.

use crate::types::Texture;
use amethyst_assets::{Asset, Handle};
use amethyst_core::ecs::prelude::DenseVecStorage;
use rendy::hal::Backend;

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
pub struct Material<B: Backend> {
    /// Alpha cutoff: the value at which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Diffuse map.
    pub albedo: Handle<Texture<B>>,
    /// Emission map.
    pub emission: Handle<Texture<B>>,
    /// Normal map.
    pub normal: Handle<Texture<B>>,
    /// Metallic-roughness map. (B channel metallic, G channel roughness)
    pub metallic_roughness: Handle<Texture<B>>,
    /// Ambient occlusion map.
    pub ambient_occlusion: Handle<Texture<B>>,
    /// Cavity map.
    pub cavity: Handle<Texture<B>>,
    /// Texture offset
    pub uv_offset: TextureOffset,
}

impl<B: Backend> Asset for Material<B> {
    const NAME: &'static str = "renderer::Material";
    type Data = Self;
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Clone)]
pub struct MaterialDefaults<B: Backend>(pub Material<B>);
