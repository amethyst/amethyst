//! Physically-based material.

use tex::Texture;

/// Material struct.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// Diffuse map.
    pub albedo: Texture,
    /// Emission map.
    pub emission: Texture,
    /// Normal map.
    pub normal: Texture,
    /// Metallic map.
    pub metallic: Texture,
    /// Reflectance value.
    pub reflectance: f32,
    /// Roughness map.
    pub roughness: Texture,
}
