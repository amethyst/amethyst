use crate::{
    formats::texture::TexturePrefab,
    mtl::{Material, MaterialDefaults, TextureOffset},
    rendy::hal::Backend,
    transparent::Transparent,
    types::Texture,
};
use amethyst_assets::{AssetStorage, Format, Handle, Loader, PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_error::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// `PrefabData` for loading `Material`s
///
/// ### Type parameters:
///
/// - `B`: `Backend` type parameter for `Material<B>` and `Texture<B>`
/// - `F`: `Format` to use for loading `Texture`s
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[serde(bound(deserialize = "TexturePrefab<B, F>: Deserialize<'de>"))]
pub struct MaterialPrefab<B: Backend, F>
where
    F: Format<Texture<B>>,
    F::Options: Clone + Debug + Serialize + for<'d> Deserialize<'d>,
{
    /// Diffuse map.
    pub albedo: Option<TexturePrefab<B, F>>,
    /// Diffuse texture offset
    pub albedo_offset: TextureOffset,
    /// Emission map.
    pub emission: Option<TexturePrefab<B, F>>,
    /// Emission texture offset
    pub emission_offset: TextureOffset,
    /// Normal map.
    pub normal: Option<TexturePrefab<B, F>>,
    /// Normal texture offset
    pub normal_offset: TextureOffset,
    /// Metallic map.
    pub metallic: Option<TexturePrefab<B, F>>,
    /// Metallic texture offset
    pub metallic_offset: TextureOffset,
    /// Roughness map.
    pub roughness: Option<TexturePrefab<B, F>>,
    /// Roughness texture offset
    pub roughness_offset: TextureOffset,
    /// Ambient occlusion map.
    pub ambient_occlusion: Option<TexturePrefab<B, F>>,
    /// Ambient occlusion texture offset
    pub ambient_occlusion_offset: TextureOffset,
    /// Caveat map.
    pub caveat: Option<TexturePrefab<B, F>>,
    /// Caveat texture offset
    pub caveat_offset: TextureOffset,
    /// Set material as `Transparent`
    pub transparent: bool,
    /// Alpha cutoff: the value below which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Clone handle only
    #[serde(skip)]
    handle: Option<Handle<Material<B>>>,
}

impl<B: Backend, F> Default for MaterialPrefab<B, F>
where
    F: Format<Texture<B>>,
    F::Options: Clone + Debug + Serialize + for<'d> Deserialize<'d>,
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
            handle: None,
        }
    }
}

fn load_handle<B: Backend, F>(
    prefab: &Option<TexturePrefab<B, F>>,
    def: &Handle<Texture<B>>,
) -> Handle<Texture<B>>
where
    F: Format<Texture<B>> + Sync + Clone,
    F::Options: Clone,
{
    prefab
        .as_ref()
        .and_then(|tp| match tp {
            TexturePrefab::Handle(h) => Some(h.clone()),
            _ => None,
        })
        .unwrap_or_else(|| def.clone())
}

impl<'a, B: Backend, F> PrefabData<'a> for MaterialPrefab<B, F>
where
    F: Format<Texture<B>> + Sync + Clone,
    F::Options: Clone + Debug + Serialize + for<'d> Deserialize<'d>,
{
    type SystemData = (
        WriteStorage<'a, Handle<Material<B>>>,
        WriteStorage<'a, Transparent>,
        ReadExpect<'a, MaterialDefaults<B>>,
        <TexturePrefab<B, F> as PrefabData<'a>>::SystemData,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Material<B>>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), Error> {
        let &mut (ref mut material, ref mut transparent, _, _, _, _) = system_data;
        material.insert(entity, self.handle.as_ref().unwrap().clone())?;
        if self.transparent {
            transparent.insert(entity, Transparent)?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let &mut (_, _, ref mat_default, ref mut tp_data, ref loader, ref storage) = system_data;
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

        if self.handle.is_none() {
            let mtl = Material {
                albedo: load_handle(&self.albedo, &mat_default.0.albedo),
                albedo_offset: self.albedo_offset.clone(),
                emission: load_handle(&self.emission, &mat_default.0.emission),
                emission_offset: self.emission_offset.clone(),
                normal: load_handle(&self.normal, &mat_default.0.normal),
                normal_offset: self.normal_offset.clone(),
                metallic: load_handle(&self.metallic, &mat_default.0.metallic),
                metallic_offset: self.metallic_offset.clone(),
                roughness: load_handle(&self.roughness, &mat_default.0.roughness),
                roughness_offset: self.roughness_offset.clone(),
                ambient_occlusion: load_handle(
                    &self.ambient_occlusion,
                    &mat_default.0.ambient_occlusion,
                ),
                ambient_occlusion_offset: self.ambient_occlusion_offset.clone(),
                caveat: load_handle(&self.caveat, &mat_default.0.caveat),
                caveat_offset: self.caveat_offset.clone(),
                alpha_cutoff: self.alpha_cutoff,
            };

            self.handle
                .replace(loader.load_from_data(mtl, progress, storage));
            ret = true;
        }

        Ok(ret)
    }
}
