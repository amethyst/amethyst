use amethyst_assets::{Format, PrefabData, PrefabError, ProgressCounter};
use amethyst_core::specs::prelude::{Entity, ReadExpect, WriteStorage};

use serde::{Deserialize, Serialize};

use crate::{
    mtl::{Material, MaterialDefaults},
    transparent::Transparent,
};

use crate::{Texture, TextureMetadata, TextureOffset, TexturePrefab, TextureView};

/// Encapsulates texture + texture offset for material prefabs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextureViewPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    /// Texture prefab
    pub texture: TexturePrefab<F>,
    /// Texture offset
    pub offset: TextureOffset,
}

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
    pub albedo: Option<TextureViewPrefab<F>>,
    /// Emission map.
    pub emission: Option<TextureViewPrefab<F>>,
    /// Normal map.
    pub normal: Option<TextureViewPrefab<F>>,
    /// Metallic map.
    pub metallic: Option<TextureViewPrefab<F>>,
    /// Roughness map.
    pub roughness: Option<TextureViewPrefab<F>>,
    /// Ambient occlusion map.
    pub ambient_occlusion: Option<TextureViewPrefab<F>>,
    /// Caveat map.
    pub caveat: Option<TextureViewPrefab<F>>,
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
            emission: None,
            normal: None,
            metallic: None,
            roughness: None,
            ambient_occlusion: None,
            caveat: None,
            transparent: false,
            alpha_cutoff: 0.01,
        }
    }
}

fn load_texture_view<F>(
    entity: Entity,
    prefab: &Option<TextureViewPrefab<F>>,
    tp_data: &mut <TexturePrefab<F> as PrefabData<'_>>::SystemData,
    def: &TextureView,
) -> TextureView
where
    F: Format<Texture, Options = TextureMetadata> + Sync + Clone,
{
    TextureView {
        texture: prefab
            .as_ref()
            .and_then(|p| p.texture.add_to_entity(entity, tp_data, &[]).ok())
            .unwrap_or_else(|| def.texture.clone()),
        offset: prefab
            .as_ref()
            .map(|p| p.offset.clone())
            .unwrap_or_else(|| TextureOffset::default()),
    }
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
            albedo: load_texture_view(entity, &self.albedo, tp_data, &mat_default.0.albedo),
            emission: load_texture_view(entity, &self.emission, tp_data, &mat_default.0.emission),
            normal: load_texture_view(entity, &self.normal, tp_data, &mat_default.0.normal),
            metallic: load_texture_view(entity, &self.metallic, tp_data, &mat_default.0.metallic),
            roughness: load_texture_view(
                entity,
                &self.roughness,
                tp_data,
                &mat_default.0.roughness,
            ),
            ambient_occlusion: load_texture_view(
                entity,
                &self.ambient_occlusion,
                tp_data,
                &mat_default.0.ambient_occlusion,
            ),
            caveat: load_texture_view(entity, &self.caveat, tp_data, &mat_default.0.caveat),
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
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.emission {
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.normal {
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.metallic {
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.roughness {
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.ambient_occlusion {
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.caveat {
            if texture.texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        Ok(ret)
    }
}
