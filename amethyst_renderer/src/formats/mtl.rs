use amethyst_assets::{Format, Handle, PrefabData, PrefabError, ProgressCounter};
use amethyst_core::specs::prelude::{Entity, ReadExpect, WriteStorage};

use serde::{Deserialize, Serialize};

use crate::{
    mtl::{Material, MaterialDefaults, TextureOffset},
    transparent::Transparent,
};

use super::{Texture, TextureMetadata, TexturePrefab};

/// `PrefabData` for loading `Material`s
///
/// ### Type parameters:
///
/// - `F`: `Format` to use for loading `Texture`s
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MaterialPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    /// Diffuse map.
    pub albedo: Option<TexturePrefab<F>>,
    /// Diffuse texture offset
    pub albedo_offset: TextureOffset,
    /// Emission map.
    pub emission: Option<TexturePrefab<F>>,
    /// Emission texture offset
    pub emission_offset: TextureOffset,
    /// Normal map.
    pub normal: Option<TexturePrefab<F>>,
    /// Normal texture offset
    pub normal_offset: TextureOffset,
    /// Metallic map.
    pub metallic: Option<TexturePrefab<F>>,
    /// Metallic texture offset
    pub metallic_offset: TextureOffset,
    /// Roughness map.
    pub roughness: Option<TexturePrefab<F>>,
    /// Roughness texture offset
    pub roughness_offset: TextureOffset,
    /// Ambient occlusion map.
    pub ambient_occlusion: Option<TexturePrefab<F>>,
    /// Ambient occlusion texture offset
    pub ambient_occlusion_offset: TextureOffset,
    /// Caveat map.
    pub caveat: Option<TexturePrefab<F>>,
    /// Caveat texture offset
    pub caveat_offset: TextureOffset,
    /// Set material as `Transparent`
    pub transparent: bool,
    /// Alpha cutoff: the value below which we do not draw the pixel
    pub alpha_cutoff: f32,
}

impl<F> Default for MaterialPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    fn default() -> Self {
        MaterialPrefab {
            albedo: None,
            albedo_offset: TextureOffset::default(),
            emission: None,
            emission_offset: TextureOffset::default(),
            normal: None,
            normal_offset: TextureOffset::default(),
            metallic: None,
            metallic_offset: TextureOffset::default(),
            roughness: None,
            roughness_offset: TextureOffset::default(),
            ambient_occlusion: None,
            ambient_occlusion_offset: TextureOffset::default(),
            caveat: None,
            caveat_offset: TextureOffset::default(),
            transparent: false,
            alpha_cutoff: 0.01,
        }
    }
}

fn load_handle<F>(
    entity: Entity,
    prefab: &Option<TexturePrefab<F>>,
    tp_data: &mut <TexturePrefab<F> as PrefabData<'_>>::SystemData,
    def: &Handle<Texture>,
) -> Handle<Texture>
where
    F: Format<Texture, Options = TextureMetadata> + Sync + Clone,
{
    prefab
        .as_ref()
        .and_then(|tp| tp.add_to_entity(entity, tp_data, &[]).ok())
        .unwrap_or_else(|| def.clone())
}

impl<'a, F> PrefabData<'a> for MaterialPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata> + Sync + Clone,
{
    type SystemData = (
        WriteStorage<'a, Material>,
        ReadExpect<'a, MaterialDefaults>,
        <TexturePrefab<F> as PrefabData<'a>>::SystemData,
        WriteStorage<'a, Transparent>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        let &mut (ref mut material, ref mat_default, ref mut tp_data, ref mut transparent) =
            system_data;
        let mtl = Material {
            albedo: load_handle(entity, &self.albedo, tp_data, &mat_default.0.albedo),
            albedo_offset: self.albedo_offset.clone(),
            emission: load_handle(entity, &self.emission, tp_data, &mat_default.0.emission),
            emission_offset: self.emission_offset.clone(),
            normal: load_handle(entity, &self.normal, tp_data, &mat_default.0.normal),
            normal_offset: self.normal_offset.clone(),
            metallic: load_handle(entity, &self.metallic, tp_data, &mat_default.0.metallic),
            metallic_offset: self.metallic_offset.clone(),
            roughness: load_handle(entity, &self.roughness, tp_data, &mat_default.0.roughness),
            roughness_offset: self.roughness_offset.clone(),
            ambient_occlusion: load_handle(
                entity,
                &self.ambient_occlusion,
                tp_data,
                &mat_default.0.ambient_occlusion,
            ),
            ambient_occlusion_offset: self.ambient_occlusion_offset.clone(),
            caveat: load_handle(entity, &self.caveat, tp_data, &mat_default.0.caveat),
            caveat_offset: self.caveat_offset.clone(),
            alpha_cutoff: self.alpha_cutoff,
        };
        material.insert(entity, mtl)?;
        if self.transparent {
            transparent.insert(entity, Transparent)?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, PrefabError> {
        let &mut (_, _, ref mut tp_data, _) = system_data;
        let mut ret = false;
        if let Some(ref mut texture) = self.albedo {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.emission {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.normal {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.metallic {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.roughness {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.ambient_occlusion {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.caveat {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        Ok(ret)
    }
}
