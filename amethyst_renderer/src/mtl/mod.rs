//! Physically-based material.

use color::Rgba;
use error::Result;
use tex::Texture;
use types::Factory;

/// Material struct.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Material {
    /// Diffuse map.
    pub albedo: Texture,
    /// Emission map.
    pub emission: Texture,
    /// Normal map.
    pub normal: Texture,
    /// Metallic map.
    pub metallic: Texture,
    /// Reflectance map.
    pub reflectance: Texture,
    /// Roughness map.
    pub roughness: Texture,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TextureKind {
    Constant(Rgba),
    Map(Texture),
}

impl TextureKind {
    pub fn into_texture(self, fac: &mut Factory) -> Result<Texture> {
        match self {
            TextureKind::Constant(c) => Texture::from_color_val(c).finish(fac),
            TextureKind::Map(tex) => Ok(tex),
        }
    }
}

/// Builds new materials.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialBuilder {
    albedo: TextureKind,
    emission: TextureKind,
    metallic: TextureKind,
    normal: TextureKind,
    reflectance: TextureKind,
    roughness: TextureKind,
}

impl MaterialBuilder {
    /// Creates a new material builder.
    pub fn new() -> Self {
        MaterialBuilder {
            albedo: TextureKind::Constant([0.0, 0.0, 0.0, 1.0].into()),
            emission: TextureKind::Constant([0.0; 4].into()),
            metallic: TextureKind::Constant([0.0; 4].into()),
            normal: TextureKind::Constant([0.0, 0.0, 1.0, 1.0].into()),
            reflectance: TextureKind::Constant([0.0; 4].into()),
            roughness: TextureKind::Constant([0.5, 0.5, 0.5, 0.5].into()),
        }
    }

    /// Sets the albedo to an existing texture map.
    pub fn with_albedo(mut self, tex: &Texture) -> Self {
        self.albedo = TextureKind::Map(tex.clone());
        self
    }

    /// Sets the emission to an existing texture map.
    pub fn with_emission(mut self, tex: &Texture) -> Self {
        self.emission = TextureKind::Map(tex.clone());
        self
    }

    /// Sets the normal to an existing texture map.
    pub fn with_normal(mut self, tex: &Texture) -> Self {
        self.normal = TextureKind::Map(tex.clone());
        self
    }

    /// Sets the reflectance to an existing texture map.
    pub fn with_reflectance(mut self, tex: &Texture) -> Self {
        self.reflectance = TextureKind::Map(tex.clone());
        self
    }

    /// Sets the roughness to an existing texture map.
    pub fn with_roughness(mut self, tex: &Texture) -> Self {
        self.roughness = TextureKind::Map(tex.clone());
        self
    }

    /// Builds and returns the new material.
    pub(crate) fn finish(self, fac: &mut Factory) -> Result<Material> {
        Ok(Material {
            albedo: self.albedo.into_texture(fac)?,
            emission: self.emission.into_texture(fac)?,
            normal: self.normal.into_texture(fac)?,
            metallic: self.metallic.into_texture(fac)?,
            reflectance: self.reflectance.into_texture(fac)?,
            roughness: self.roughness.into_texture(fac)?,
        })
    }
}
