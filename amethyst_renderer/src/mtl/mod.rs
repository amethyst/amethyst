//! Physically-based material.

use specs::{Component, DenseVecStorage};

use tex::TextureHandle;
use tex_animation::SpriteSheetAnimation;

/// Material struct.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Material {
    /// Diffuse map.
    pub albedo: TextureHandle,
    /// Optional animation for albedo
    pub albedo_animation: Option<SpriteSheetAnimation>,
    /// Emission map.
    pub emission: TextureHandle,
    /// Optional animation for emission
    pub emission_animation: Option<SpriteSheetAnimation>,
    /// Normal map.
    pub normal: TextureHandle,
    /// Optional animation for normal
    pub normal_animation: Option<SpriteSheetAnimation>,
    /// Metallic map.
    pub metallic: TextureHandle,
    /// Optional animation for metallic
    pub metallic_animation: Option<SpriteSheetAnimation>,
    /// Roughness map.
    pub roughness: TextureHandle,
    /// Optional animation for roughness
    pub roughness_animation: Option<SpriteSheetAnimation>,
    /// Ambient occlusion map.
    pub ambient_occlusion: TextureHandle,
    /// Optional animation for ambient occlusion
    pub ambient_occlusion_animation: Option<SpriteSheetAnimation>,
    /// Caveat map.
    pub caveat: TextureHandle,
    /// Optional animation for caveat
    pub caveat_animation: Option<SpriteSheetAnimation>,
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
