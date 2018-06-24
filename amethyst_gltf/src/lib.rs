extern crate amethyst_animation as animation;
extern crate amethyst_assets as assets;
extern crate amethyst_core as core;
extern crate amethyst_renderer as renderer;
extern crate base64;
extern crate fnv;
extern crate gfx;
extern crate gltf;
extern crate hibitset;
extern crate imagefmt;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate mikktspace;
#[macro_use]
extern crate serde;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

pub use format::GltfSceneFormat;

use std::ops::Range;

use animation::{AnimatablePrefab, SkinnablePrefab};
use assets::{Handle, Prefab, PrefabData, PrefabLoaderSystem, ProgressCounter};
use core::cgmath::{Array, EuclideanSpace, Point3, Vector3};
use core::specs::error::Error;
use core::specs::prelude::{Component, DenseVecStorage, Entity, WriteStorage};
use core::transform::Transform;
use renderer::{MaterialPrefab, Mesh, MeshData, TextureFormat};

mod format;

/// Load `GltfSceneAsset`s
pub type GltfSceneLoaderSystem = PrefabLoaderSystem<GltfPrefab>;

/// Gltf scene asset as returned by the `GltfSceneFormat`
pub type GltfSceneAsset = Prefab<GltfPrefab>;

/// `PrefabData` for loading Gltf files.
#[derive(Debug, Clone, Default)]
pub struct GltfPrefab {
    /// `Transform` will almost always be placed, the only exception is for the main `Entity` for
    /// certain scenarios (based on the data in the Gltf file)
    pub transform: Option<Transform>,
    /// `MeshData` is placed on all `Entity`s with graphics primitives
    pub mesh: Option<MeshData>,
    /// Mesh handle after sub asset loading is done
    pub mesh_handle: Option<Handle<Mesh>>,
    /// `MeshData` is placed on all `Entity`s with graphics primitives with material
    pub material: Option<MaterialPrefab<TextureFormat>>,
    /// Loaded animations, if applicable, will always only be placed on the main `Entity`
    pub animatable: Option<AnimatablePrefab<usize, Transform>>,
    /// Skin data is placed on `Entity`s involved in the skin, skeleton or graphical primitives
    /// using the skin
    pub skinnable: Option<SkinnablePrefab>,
    /// Node extent
    pub extent: Option<GltfNodeExtent>,
}

impl GltfPrefab {
    /// Move the scene so the center of the bounding box is at the given `target` location.
    pub fn move_to(&mut self, target: Point3<f32>) {
        if let Some(ref extent) = self.extent {
            self.transform
                .get_or_insert_with(Transform::default)
                .translation += target - extent.centroid();
        }
    }

    /// Scale the scene to a specific max size
    pub fn scale_to(&mut self, max_distance: f32) {
        if let Some(ref extent) = self.extent {
            let distance = extent.distance();
            let max = distance.x.max(distance.y).max(distance.z);
            let scale = max_distance / max;
            self.transform.get_or_insert_with(Transform::default).scale =
                Vector3::from_value(scale);
        }
    }
}

#[derive(Clone, Debug)]
pub struct GltfNodeExtent {
    pub start: Point3<f32>,
    pub end: Point3<f32>,
}

impl Default for GltfNodeExtent {
    fn default() -> Self {
        Self {
            start: Point3::from_value(std::f32::MAX),
            end: Point3::from_value(std::f32::MIN),
        }
    }
}

impl GltfNodeExtent {
    pub fn extend_range(&mut self, other: &Range<[f32; 3]>) {
        for i in 0..3 {
            if other.start[i] < self.start[i] {
                self.start[i] = other.start[i];
            }
            if other.end[i] > self.end[i] {
                self.end[i] = other.end[i];
            }
        }
    }

    pub fn extend(&mut self, other: &GltfNodeExtent) {
        for i in 0..3 {
            if other.start[i] < self.start[i] {
                self.start[i] = other.start[i];
            }
            if other.end[i] > self.end[i] {
                self.end[i] = other.end[i];
            }
        }
    }

    pub fn centroid(&self) -> Point3<f32> {
        (self.start + self.end.to_vec()) / 2.
    }

    pub fn distance(&self) -> Vector3<f32> {
        self.end - self.start
    }

    pub fn valid(&self) -> bool {
        for i in 0..3 {
            if self.start[i] > self.end[i] {
                return false;
            }
        }
        true
    }
}

impl From<Range<[f32; 3]>> for GltfNodeExtent {
    fn from(range: Range<[f32; 3]>) -> Self {
        GltfNodeExtent {
            start: Point3::from(range.start),
            end: Point3::from(range.end),
        }
    }
}

impl Component for GltfNodeExtent {
    type Storage = DenseVecStorage<Self>;
}

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GltfSceneOptions {
    /// Generate texture coordinates if none exist in the Gltf file
    pub generate_tex_coords: (f32, f32),
    /// Load animation data from the Gltf file
    pub load_animations: bool,
    /// Flip the v coordinate for all texture coordinates
    pub flip_v_coord: bool,
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
        WriteStorage<'a, GltfNodeExtent>,
    );
    type Result = ();

    fn load_prefab(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        let (
            ref mut transforms,
            ref mut meshes,
            ref mut materials,
            ref mut animatables,
            ref mut skinnables,
            ref mut extents,
        ) = system_data;
        if let Some(ref transform) = self.transform {
            transform.load_prefab(entity, transforms, entities)?;
        }
        if let Some(ref mesh) = self.mesh_handle {
            meshes.1.insert(entity, mesh.clone())?;
        }
        if let Some(ref material) = self.material {
            material.load_prefab(entity, materials, entities)?;
        }
        if let Some(ref animatable) = self.animatable {
            animatable.load_prefab(entity, animatables, entities)?;
        }
        if let Some(ref skinnable) = self.skinnable {
            skinnable.load_prefab(entity, skinnables, entities)?;
        }
        if let Some(ref extent) = self.extent {
            extents.insert(entity, extent.clone())?;
        }
        Ok(())
    }

    fn trigger_sub_loading(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (_, ref mut meshes, ref mut materials, ref mut animatables, _, _) = system_data;
        let mut ret = false;
        if let Some(ref mesh) = self.mesh {
            self.mesh_handle = Some(meshes.0.load_from_data(
                mesh.clone(),
                &mut *progress,
                &meshes.2,
            ));
            ret = true;
        }
        if let Some(ref mut material) = self.material {
            if material.trigger_sub_loading(progress, materials)? {
                ret = true;
            }
        }
        if let Some(ref mut animatable) = self.animatable {
            if animatable.trigger_sub_loading(progress, animatables)? {
                ret = true;
            }
        }
        Ok(ret)
    }
}
