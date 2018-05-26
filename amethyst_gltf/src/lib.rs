extern crate amethyst_animation as animation;
extern crate amethyst_assets as assets;
extern crate amethyst_core as core;
extern crate amethyst_renderer as renderer;
extern crate base64;
extern crate fnv;
extern crate gfx;
extern crate gltf;
extern crate gltf_utils;
extern crate hibitset;
extern crate imagefmt;
extern crate itertools;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use format::GltfSceneFormat;

use animation::{AnimatablePrefab, SkinnablePrefab};
use assets::{Prefab, PrefabData, PrefabLoaderSystem};
use core::specs::error::Error;
use core::specs::prelude::Entity;
use core::transform::Transform;
use renderer::{MaterialPrefab, MeshData, TextureFormat};

mod format;

/// Load `GltfSceneAsset`s
pub type GltfSceneLoaderSystem = PrefabLoaderSystem<GltfPrefab>;

/// Gltf scene asset as returned by the `GltfSceneFormat`
pub type GltfSceneAsset = Prefab<GltfPrefab>;

/// `PrefabData` for loading Gltf files.
#[derive(Clone, Default)]
pub struct GltfPrefab {
    /// `Transform` will almost always be placed, the only exception is for the main `Entity` for
    /// certain scenarios (based on the data in the Gltf file)
    transform: Option<Transform>,
    /// `MeshData` is placed on all `Entity`s with graphics primitives
    mesh: Option<MeshData>,
    /// `MeshData` is placed on all `Entity`s with graphics primitives with material
    material: Option<MaterialPrefab<TextureFormat>>,
    /// Loaded animations, if applicable, will always only be placed on the main `Entity`
    animatable: Option<AnimatablePrefab<usize, Transform>>,
    /// Skin data is placed on `Entity`s involved in the skin, skeleton or graphical primitives
    /// using the skin
    skinnable: Option<SkinnablePrefab>,
}

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GltfSceneOptions {
    /// Generate texture coordinates if none exist in the Gltf file
    pub generate_tex_coords: Option<(f32, f32)>,
    /// Load animation data from the Gltf file
    pub load_animations: bool,
    /// Flip the v coordinate for all texture coordinates
    pub flip_v_coord: bool,
    /// Will translate the main `Entity` so that the extent centroid of the scene is place in the
    /// origin
    pub move_to_origin: bool,
    /// Load the given scene index, if not supplied will either load the default scene (if set),
    /// or the first scene (only if there is only one scene, otherwise an `Error` will be returned).
    pub scene_index: Option<usize>,
}

impl<'a> PrefabData<'a> for GltfPrefab {
    type SystemData = (
        <Transform as PrefabData<'a>>::SystemData,
        <MeshData as PrefabData<'a>>::SystemData,
        <MaterialPrefab<TextureFormat> as PrefabData<'a>>::SystemData,
        <AnimatablePrefab<usize, Transform> as PrefabData<'a>>::SystemData,
        <SkinnablePrefab as PrefabData<'a>>::SystemData,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        if let Some(ref transform) = self.transform {
            transform.load_prefab(entity, &mut system_data.0, entities)?;
        }
        if let Some(ref mesh) = self.mesh {
            mesh.load_prefab(entity, &mut system_data.1, entities)?;
        }
        if let Some(ref material) = self.material {
            material.load_prefab(entity, &mut system_data.2, entities)?;
        }
        if let Some(ref animatable) = self.animatable {
            animatable.load_prefab(entity, &mut system_data.3, entities)?;
        }
        if let Some(ref skinnable) = self.skinnable {
            skinnable.load_prefab(entity, &mut system_data.4, entities)?;
        }
        Ok(())
    }
}
