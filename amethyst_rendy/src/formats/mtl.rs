//! Material type implementation.

use crate::{
    formats::texture::TexturePrefab,
    mtl::{Material, MaterialDefaults, TextureOffset},
    transparent::Transparent,
    types::Texture,
};
use amethyst_assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter};
use amethyst_core::ecs::prelude::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_error::Error;
use serde::{Deserialize, Serialize};

/// `PrefabData` for loading `Material`s
#[derive(Debug, Deserialize, Serialize)]
#[serde(default, bound = "")]
pub struct MaterialPrefab {
    /// Diffuse map.
    pub albedo: Option<TexturePrefab>,
    /// Emission map.
    pub emission: Option<TexturePrefab>,
    /// Normal map.
    pub normal: Option<TexturePrefab>,
    /// Metallic-roughness map. (B channel metallic, G channel roughness)
    pub metallic_roughness: Option<TexturePrefab>,
    /// Ambient occlusion map.
    pub ambient_occlusion: Option<TexturePrefab>,
    /// Cavity map.
    pub cavity: Option<TexturePrefab>,
    /// Texture offset.
    pub uv_offset: TextureOffset,
    /// Set material as `Transparent`
    pub transparent: bool,
    /// Alpha cutoff: the value below which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Clone handle only
    #[serde(skip)]
    handle: Option<Handle<Material>>,
}

impl MaterialPrefab {
    /// Clone the loaded material prefab to a new instance with an independent transparency value.
    pub fn clone_loaded(&self) -> Self {
        assert!(self.handle.is_some());

        Self {
            handle: self.handle.clone(),
            transparent: self.transparent,
            ..Self::default()
        }
    }
}

impl Default for MaterialPrefab {
    fn default() -> Self {
        MaterialPrefab {
            albedo: None,
            emission: None,
            normal: None,
            metallic_roughness: None,
            ambient_occlusion: None,
            cavity: None,
            uv_offset: TextureOffset::default(),
            transparent: false,
            alpha_cutoff: std::f32::MIN_POSITIVE,
            handle: None,
        }
    }
}

fn load_handle(prefab: &Option<TexturePrefab>, def: &Handle<Texture>) -> Handle<Texture> {
    prefab
        .as_ref()
        .and_then(|tp| match tp {
            TexturePrefab::Handle(h) => Some(h.clone()),
            _ => None,
        })
        .unwrap_or_else(|| def.clone())
}

impl<'a> PrefabData<'a> for MaterialPrefab {
    type SystemData = (
        WriteStorage<'a, Handle<Material>>,
        WriteStorage<'a, Transparent>,
        ReadExpect<'a, MaterialDefaults>,
        <TexturePrefab as PrefabData<'a>>::SystemData,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<Material>>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _: &[Entity],
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
        if let Some(ref mut texture) = self.metallic_roughness {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.ambient_occlusion {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }
        if let Some(ref mut texture) = self.cavity {
            if texture.load_sub_assets(progress, tp_data)? {
                ret = true;
            }
        }

        if self.handle.is_none() {
            let mtl = Material {
                albedo: load_handle(&self.albedo, &mat_default.0.albedo),
                emission: load_handle(&self.emission, &mat_default.0.emission),
                normal: load_handle(&self.normal, &mat_default.0.normal),
                metallic_roughness: load_handle(
                    &self.metallic_roughness,
                    &mat_default.0.metallic_roughness,
                ),
                ambient_occlusion: load_handle(
                    &self.ambient_occlusion,
                    &mat_default.0.ambient_occlusion,
                ),
                cavity: load_handle(&self.cavity, &mat_default.0.cavity),
                uv_offset: self.uv_offset.clone(),
                alpha_cutoff: self.alpha_cutoff,
            };

            self.handle
                .replace(loader.load_from_data(mtl, progress, storage));
            ret = true;
        }

        Ok(ret)
    }
}
