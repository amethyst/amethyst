//! Physically-based material.

use gfx::traits::Pod;

use specs::{Component, DenseVecStorage};

use error::Result;
use tex::{Texture, TextureBuilder};
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
    /// Roughness map.
    pub roughness: Texture,
    /// Ambient occlusion map.
    pub ambient_occlusion: Texture,
    /// Caveat map.
    pub caveat: Texture,
}

/// Builds new materials.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC> {
    albedo: TextureBuilder<DA, TA>,
    emission: TextureBuilder<DE, TE>,
    normal: TextureBuilder<DN, TN>,
    metallic: TextureBuilder<DM, TM>,
    roughness: TextureBuilder<DR, TR>,
    ambient_occlusion: TextureBuilder<DO, TO>,
    caveat: TextureBuilder<DC, TC>,
}

impl
    MaterialBuilder<
        [u8; 4],
        u8,
        [u8; 4],
        u8,
        [u8; 4],
        u8,
        [u8; 4],
        u8,
        [u8; 4],
        u8,
        [u8; 4],
        u8,
        [u8; 4],
        u8,
    > {
    /// Creates a new material builder.
    pub fn new() -> Self {
        MaterialBuilder {
            albedo: TextureBuilder::from_color_val([0.0, 0.0, 0.5, 1.0]),
            emission: TextureBuilder::from_color_val([0.0; 4]),
            normal: TextureBuilder::from_color_val([0.5, 0.5, 1.0, 1.0]),
            metallic: TextureBuilder::from_color_val([0.0; 4]),
            roughness: TextureBuilder::from_color_val([0.5; 4]),
            ambient_occlusion: TextureBuilder::from_color_val([1.0; 4]),
            caveat: TextureBuilder::from_color_val([1.0; 4]),
        }
    }
}

impl<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>
    MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC> {
    /// Sets the albedo to an existing texture map.
    pub fn with_albedo<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<Y, U, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC> {
        MaterialBuilder {
            albedo: tex,
            emission: self.emission,
            normal: self.normal,
            metallic: self.metallic,
            roughness: self.roughness,
            ambient_occlusion: self.ambient_occlusion,
            caveat: self.caveat,
        }
    }

    /// Sets the emission to an existing texture map.
    pub fn with_emission<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<DA, TA, Y, U, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC> {
        MaterialBuilder {
            albedo: self.albedo,
            emission: tex,
            normal: self.normal,
            metallic: self.metallic,
            roughness: self.roughness,
            ambient_occlusion: self.ambient_occlusion,
            caveat: self.caveat,
        }
    }

    /// Sets the normal to an existing texture map.
    pub fn with_normal<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<DA, TA, DE, TE, Y, U, DM, TM, DR, TR, DO, TO, DC, TC> {
        MaterialBuilder {
            albedo: self.albedo,
            emission: self.emission,
            normal: tex,
            metallic: self.metallic,
            roughness: self.roughness,
            ambient_occlusion: self.ambient_occlusion,
            caveat: self.caveat,
        }
    }

    /// Sets the metallic to an existing texture map.
    pub fn with_metallic<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<DA, TA, DE, TE, DN, TN, Y, U, DR, TR, DO, TO, DC, TC> {
        MaterialBuilder {
            albedo: self.albedo,
            emission: self.emission,
            normal: self.normal,
            metallic: tex,
            roughness: self.roughness,
            ambient_occlusion: self.ambient_occlusion,
            caveat: self.caveat,
        }
    }

    /// Sets the roughness to an existing texture map.
    pub fn with_roughness<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, Y, U, DO, TO, DC, TC> {
        MaterialBuilder {
            albedo: self.albedo,
            emission: self.emission,
            normal: self.normal,
            metallic: self.metallic,
            roughness: tex,
            ambient_occlusion: self.ambient_occlusion,
            caveat: self.caveat,
        }
    }

    /// Sets the ambient_occlusion to an existing texture map.
    pub fn with_ambient_occlusion<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, Y, U, DC, TC> {
        MaterialBuilder {
            albedo: self.albedo,
            emission: self.emission,
            normal: self.normal,
            metallic: self.metallic,
            roughness: self.roughness,
            ambient_occlusion: tex,
            caveat: self.caveat,
        }
    }

    /// Sets the caveat to an existing texture map.
    pub fn with_caveat<Y, U>(
        self,
        tex: TextureBuilder<Y, U>,
    ) -> MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, Y, U> {
        MaterialBuilder {
            albedo: self.albedo,
            emission: self.emission,
            normal: self.normal,
            metallic: self.metallic,
            roughness: self.roughness,
            ambient_occlusion: self.ambient_occlusion,
            caveat: tex,
        }
    }

    /// Builds and returns the new material.
    pub fn build(self, fac: &mut Factory) -> Result<Material>
    where
        DA: AsRef<[TA]>,
        TA: Pod + Copy,
        DE: AsRef<[TE]>,
        TE: Pod + Copy,
        DN: AsRef<[TN]>,
        TN: Pod + Copy,
        DM: AsRef<[TM]>,
        TM: Pod + Copy,
        DR: AsRef<[TR]>,
        TR: Pod + Copy,
        DO: AsRef<[TO]>,
        TO: Pod + Copy,
        DC: AsRef<[TC]>,
        TC: Pod + Copy,
    {
        Ok(Material {
            albedo: self.albedo.build(fac)?,
            emission: self.emission.build(fac)?,
            normal: self.normal.build(fac)?,
            metallic: self.metallic.build(fac)?,
            roughness: self.roughness.build(fac)?,
            ambient_occlusion: self.ambient_occlusion.build(fac)?,
            caveat: self.caveat.build(fac)?,
        })
    }
}


impl Component for Material {
    type Storage = DenseVecStorage<Self>;
}
