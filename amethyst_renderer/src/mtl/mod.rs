//! Physically-based material.

use specs::{Component, DenseVecStorage};

use tex::TextureHandle;
use tex_animation::SpriteSheetAnimation;

/// A material provides textures that are used for rendering a `Mesh`.
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

/// This provides data for how spritesheets should be animated on a material.  To use this
/// create a `Material` that has the spritesheets attached in the textures, then provide how
/// those spritesheets are to be animated from this structure.
pub struct MaterialAnimation {
    /// Optional animation for albedo
    pub albedo_animation: Option<SpriteSheetAnimation>,
    /// Optional animation for emission
    pub emission_animation: Option<SpriteSheetAnimation>,
    /// Optional animation for normal
    pub normal_animation: Option<SpriteSheetAnimation>,
    /// Optional animation for metallic
    pub metallic_animation: Option<SpriteSheetAnimation>,
    /// Optional animation for roughness
    pub roughness_animation: Option<SpriteSheetAnimation>,
    /// Optional animation for ambient occlusion
    pub ambient_occlusion_animation: Option<SpriteSheetAnimation>,
    /// Optional animation for caveat
    pub caveat_animation: Option<SpriteSheetAnimation>,
}

impl Component for MaterialAnimation {
    type Storage = DenseVecStorage<Self>;
}

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Clone)]
pub struct MaterialDefaults(pub Material);
