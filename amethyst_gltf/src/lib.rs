//! A crate for loading GLTF format scenes into Amethyst

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

use serde::{Deserialize, Serialize};

pub use crate::format::GltfSceneFormat;

use std::{collections::HashMap, ops::Range};

use amethyst_animation as animation;
use amethyst_assets as assets;
use amethyst_core as core;
use amethyst_renderer as renderer;

use crate::{
    animation::{AnimatablePrefab, SkinnablePrefab},
    assets::{Handle, Prefab, PrefabData, PrefabLoaderSystem, ProgressCounter},
    core::{
        nalgebra::{Point3, Vector3},
        specs::{
            error::Error,
            prelude::{Component, DenseVecStorage, Entity, Write, WriteStorage},
        },
        transform::Transform,
        Named,
    },
    renderer::{MaterialPrefab, Mesh, MeshData, TextureFormat},
};

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
    /// `Material` is placed on all `Entity`s with graphics primitives with material
    pub material: Option<MaterialPrefab<TextureFormat>>,
    /// Loaded animations, if applicable, will always only be placed on the main `Entity`
    pub animatable: Option<AnimatablePrefab<usize, Transform>>,
    /// Skin data is placed on `Entity`s involved in the skin, skeleton or graphical primitives
    /// using the skin
    pub skinnable: Option<SkinnablePrefab>,
    /// Node extent
    pub extent: Option<GltfNodeExtent>,
    /// Node name
    pub name: Option<Named>,
    pub(crate) materials: Option<GltfMaterialSet>,
    pub(crate) material_id: Option<usize>,
}

impl GltfPrefab {
    /// Move the scene so the center of the bounding box is at the given `target` location.
    pub fn move_to(&mut self, target: Point3<f32>) {
        if let Some(ref extent) = self.extent {
            *self
                .transform
                .get_or_insert_with(Transform::default)
                .translation_mut() += target - extent.centroid();
        }
    }

    /// Scale the scene to a specific max size
    pub fn scale_to(&mut self, max_distance: f32) {
        if let Some(ref extent) = self.extent {
            let distance = extent.distance();
            let max = distance.x.max(distance.y).max(distance.z);
            let scale = max_distance / max;
            self.transform
                .get_or_insert_with(Transform::default)
                .set_scale(scale, scale, scale);
        }
    }
}

/// A GLTF node extent
#[derive(Clone, Debug)]
pub struct GltfNodeExtent {
    /// The beginning of this extent
    pub start: Point3<f32>,
    /// The end of this extent
    pub end: Point3<f32>,
}

impl Default for GltfNodeExtent {
    fn default() -> Self {
        Self {
            start: Point3::from(Vector3::from_element(std::f32::MAX)),
            end: Point3::from(Vector3::from_element(std::f32::MIN)),
        }
    }
}

impl GltfNodeExtent {
    /// Extends this to include the input range.
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

    /// Extends this to include the provided extent.
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

    /// Returns the centroid of this extent
    pub fn centroid(&self) -> Point3<f32> {
        (self.start + self.end.coords) / 2.
    }

    /// Returns the 3 dimensional distance between the start and end of this.
    pub fn distance(&self) -> Vector3<f32> {
        self.end - self.start
    }

    /// Determines if this extent is valid.
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

/// Used during gltf loading to contain the materials used from scenes in the file
#[derive(Default, Clone, Debug)]
pub struct GltfMaterialSet {
    pub(crate) materials: HashMap<usize, MaterialPrefab<TextureFormat>>,
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
        <Named as PrefabData<'a>>::SystemData,
        <MaterialPrefab<TextureFormat> as PrefabData<'a>>::SystemData,
        <AnimatablePrefab<usize, Transform> as PrefabData<'a>>::SystemData,
        <SkinnablePrefab as PrefabData<'a>>::SystemData,
        WriteStorage<'a, GltfNodeExtent>,
        // TODO make optional after prefab refactor. We need a way to pass options to decide to enable this or not, but without touching the prefab.
        WriteStorage<'a, MeshData>,
        Write<'a, GltfMaterialSet>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        let (
            ref mut transforms,
            ref mut meshes,
            ref mut names,
            ref mut materials,
            ref mut animatables,
            ref mut skinnables,
            ref mut extents,
            ref mut mesh_data,
            _,
        ) = system_data;
        if let Some(ref transform) = self.transform {
            transform.add_to_entity(entity, transforms, entities)?;
        }
        if let Some(ref mesh) = self.mesh {
            mesh_data.insert(entity, mesh.clone())?;
        }
        if let Some(ref mesh) = self.mesh_handle {
            meshes.1.insert(entity, mesh.clone())?;
        }
        if let Some(ref name) = self.name {
            name.add_to_entity(entity, names, entities)?;
        }
        if let Some(ref material) = self.material {
            material.add_to_entity(entity, materials, entities)?;
        }
        if let Some(ref animatable) = self.animatable {
            animatable.add_to_entity(entity, animatables, entities)?;
        }
        if let Some(ref skinnable) = self.skinnable {
            skinnable.add_to_entity(entity, skinnables, entities)?;
        }
        if let Some(ref extent) = self.extent {
            extents.insert(entity, extent.clone())?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (
            _,
            ref mut meshes,
            _,
            ref mut materials,
            ref mut animatables,
            _,
            _,
            _,
            ref mut mat_set,
        ) = system_data;
        let mut ret = false;
        if let Some(ref mut mats) = self.materials {
            mat_set.materials.clear();
            for (id, material) in mats.materials.iter_mut() {
                if material.load_sub_assets(progress, materials)? {
                    ret = true;
                }
                mat_set.materials.insert(*id, material.clone());
            }
        }
        if let Some(ref mesh) = self.mesh {
            self.mesh_handle = Some(meshes.0.load_from_data(
                mesh.clone(),
                &mut *progress,
                &meshes.2,
            ));
            ret = true;
        }
        match self.material_id {
            Some(material_id) => {
                if let Some(mat) = mat_set.materials.get(&material_id) {
                    self.material = Some(mat.clone());
                }
            }
            None => {
                if let Some(ref mut material) = self.material {
                    if material.load_sub_assets(progress, materials)? {
                        ret = true;
                    }
                }
            }
        }
        if let Some(ref mut animatable) = self.animatable {
            if animatable.load_sub_assets(progress, animatables)? {
                ret = true;
            }
        }
        Ok(ret)
    }
}
