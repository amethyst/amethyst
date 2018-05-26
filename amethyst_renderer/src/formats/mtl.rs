use amethyst_assets::{Format, Handle, PrefabData};
use amethyst_core::specs::error::Error as SpecsError;
use amethyst_core::specs::prelude::{Entity, ReadExpect, Write, WriteStorage};

use super::{Texture, TextureMetadata, TexturePrefab};
use mtl::{Material, MaterialDefaults, MaterialTextureSet, TextureOffset};
use transparent::Transparent;

/// `PrefabData` for loading `Material`s
///
/// ### Type parameters:
///
/// - `F`: `Format` to use for loading `Texture`s
#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MaterialPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    /// Diffuse map.
    pub albedo: Option<TexturePrefab<F>>,
    /// Diffuse texture offset
    pub albedo_offset: TextureOffset,
    /// Albedo texture animation index, for addition to `MaterialTextureSet`
    pub albedo_index: Option<usize>,
    /// Emission map.
    pub emission: Option<TexturePrefab<F>>,
    /// Emission texture offset
    pub emission_offset: TextureOffset,
    /// Emission texture animation index, for addition to `MaterialTextureSet`
    pub emission_index: Option<usize>,
    /// Normal map.
    pub normal: Option<TexturePrefab<F>>,
    /// Normal texture offset
    pub normal_offset: TextureOffset,
    /// Normal texture animation index, for addition to `MaterialTextureSet`
    pub normal_index: Option<usize>,
    /// Metallic map.
    pub metallic: Option<TexturePrefab<F>>,
    /// Metallic texture offset
    pub metallic_offset: TextureOffset,
    /// Metallic texture animation index, for addition to `MaterialTextureSet`
    pub metallic_index: Option<usize>,
    /// Roughness map.
    pub roughness: Option<TexturePrefab<F>>,
    /// Roughness texture offset
    pub roughness_offset: TextureOffset,
    /// Roughness texture animation index, for addition to `MaterialTextureSet`
    pub roughness_index: Option<usize>,
    /// Ambient occlusion map.
    pub ambient_occlusion: Option<TexturePrefab<F>>,
    /// Ambient occlusion texture offset
    pub ambient_occlusion_offset: TextureOffset,
    /// Ambient occlusion texture animation index, for addition to `MaterialTextureSet`
    pub ambient_occlusion_index: Option<usize>,
    /// Caveat map.
    pub caveat: Option<TexturePrefab<F>>,
    /// Caveat texture offset
    pub caveat_offset: TextureOffset,
    /// Caveat texture animation index, for addition to `MaterialTextureSet`
    pub caveat_index: Option<usize>,
    /// Set material as `Transparent`
    pub transparent: bool,
}

impl<F> Default for MaterialPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata>,
{
    fn default() -> Self {
        MaterialPrefab {
            albedo: None,
            albedo_offset: TextureOffset::default(),
            albedo_index: None,
            emission: None,
            emission_offset: TextureOffset::default(),
            emission_index: None,
            normal: None,
            normal_offset: TextureOffset::default(),
            normal_index: None,
            metallic: None,
            metallic_offset: TextureOffset::default(),
            metallic_index: None,
            roughness: None,
            roughness_offset: TextureOffset::default(),
            roughness_index: None,
            ambient_occlusion: None,
            ambient_occlusion_offset: TextureOffset::default(),
            ambient_occlusion_index: None,
            caveat: None,
            caveat_offset: TextureOffset::default(),
            caveat_index: None,
            transparent: false,
        }
    }
}

fn load_handle<F>(
    entity: Entity,
    index: Option<usize>,
    prefab: &Option<TexturePrefab<F>>,
    texture_set: &mut MaterialTextureSet,
    tp_data: &mut <TexturePrefab<F> as PrefabData>::SystemData,
    def: &Handle<Texture>,
) -> Handle<Texture>
where
    F: Format<Texture, Options = TextureMetadata> + Sync + Clone,
{
    index
        .and_then(|i| texture_set.handle(i))
        .unwrap_or_else(|| {
            let handle = prefab
                .as_ref()
                .and_then(|tp| tp.load_prefab(entity, tp_data, &[]).ok());
            if let (&Some(ref index), &Some(ref handle)) = (&index, &handle) {
                texture_set.insert(*index, handle.clone());
            }
            handle.unwrap_or(def.clone())
        })
}

impl<'a, F> PrefabData<'a> for MaterialPrefab<F>
where
    F: Format<Texture, Options = TextureMetadata> + Sync + Clone,
{
    type SystemData = (
        WriteStorage<'a, Material>,
        ReadExpect<'a, MaterialDefaults>,
        Write<'a, MaterialTextureSet>,
        <TexturePrefab<F> as PrefabData<'a>>::SystemData,
        WriteStorage<'a, Transparent>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), SpecsError> {
        let &mut (
            ref mut material,
            ref mat_default,
            ref mut texture_set,
            ref mut tp_data,
            ref mut transparent,
        ) = system_data;
        let mtl = Material {
            albedo: load_handle(
                entity,
                self.albedo_index,
                &self.albedo,
                texture_set,
                tp_data,
                &mat_default.0.albedo,
            ),
            albedo_offset: self.albedo_offset.clone(),
            emission: load_handle(
                entity,
                self.emission_index,
                &self.emission,
                texture_set,
                tp_data,
                &mat_default.0.emission,
            ),
            emission_offset: self.emission_offset.clone(),
            normal: load_handle(
                entity,
                self.normal_index,
                &self.normal,
                texture_set,
                tp_data,
                &mat_default.0.normal,
            ),
            normal_offset: self.normal_offset.clone(),
            metallic: load_handle(
                entity,
                self.metallic_index,
                &self.metallic,
                texture_set,
                tp_data,
                &mat_default.0.metallic,
            ),
            metallic_offset: self.metallic_offset.clone(),
            roughness: load_handle(
                entity,
                self.roughness_index,
                &self.roughness,
                texture_set,
                tp_data,
                &mat_default.0.roughness,
            ),
            roughness_offset: self.roughness_offset.clone(),
            ambient_occlusion: load_handle(
                entity,
                self.ambient_occlusion_index,
                &self.ambient_occlusion,
                texture_set,
                tp_data,
                &mat_default.0.ambient_occlusion,
            ),
            ambient_occlusion_offset: self.ambient_occlusion_offset.clone(),
            caveat: load_handle(
                entity,
                self.caveat_index,
                &self.caveat,
                texture_set,
                tp_data,
                &mat_default.0.caveat,
            ),
            caveat_offset: self.caveat_offset.clone(),
        };
        material.insert(entity, mtl)?;
        if self.transparent {
            transparent.insert(entity, Transparent)?;
        }
        Ok(())
    }
}
