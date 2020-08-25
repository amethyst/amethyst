//! A crate for loading GLTF format scenes into Amethyst

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

use amethyst_animation::{AnimatablePrefab, SkinnablePrefab};
use amethyst_assets::{
    AssetStorage, Handle, Loader, Prefab, PrefabData, PrefabLoaderSystem, PrefabLoaderSystemDesc,
    ProgressCounter,
};
use amethyst_core::{
    ecs::prelude::{Entity, Read, ReadExpect, Write, WriteStorage},
    math::{convert, Point3, Vector3},
    transform::Transform,
    Named,
};
use amethyst_error::Error;
use amethyst_rendy::{
    camera::CameraPrefab, formats::mtl::MaterialPrefab, light::LightPrefab,
    rendy::mesh::MeshBuilder, types::Mesh, visibility::BoundingSphere,
};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Range};

pub use crate::format::GltfSceneFormat;

mod error;
mod format;

/// Builds a `GltfSceneLoaderSystem`.
pub type GltfSceneLoaderSystemDesc = PrefabLoaderSystemDesc<GltfPrefab>;

/// Load `GltfSceneAsset`s
pub type GltfSceneLoaderSystem = PrefabLoaderSystem<GltfPrefab>;

/// Gltf scene asset as returned by the `GltfSceneFormat`
pub type GltfSceneAsset = Prefab<GltfPrefab>;

/// `PrefabData` for loading Gltf files.
#[derive(Debug, Default)]
pub struct GltfPrefab {
    /// `Transform` will almost always be placed, the only exception is for the main `Entity` for
    /// certain scenarios (based on the data in the Gltf file)
    pub transform: Option<Transform>,
    /// `Camera` will always be placed
    pub camera: Option<CameraPrefab>,
    /// Lights can be added to a prefab with the `KHR_lights_punctual` feature enabled
    pub light: Option<LightPrefab>,
    /// `MeshData` is placed on all `Entity`s with graphics primitives
    pub mesh: Option<MeshBuilder<'static>>,
    /// Mesh handle after sub asset loading is done
    pub mesh_handle: Option<Handle<Mesh>>,
    /// `Material` is placed on all `Entity`s with graphics primitives with material
    pub material: Option<MaterialPrefab>,
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
            let diff = convert::<_, Vector3<f32>>(target - extent.centroid());
            *self
                .transform
                .get_or_insert_with(Transform::default)
                .translation_mut() += diff;
        }
    }

    /// Scale the scene to a specific max size
    pub fn scale_to(&mut self, max_distance: f32) {
        if let Some(ref extent) = self.extent {
            let distance = extent.distance();
            let max = distance.x.max(distance.y).max(distance.z);
            let scale: f32 = max_distance / max;
            self.transform
                .get_or_insert_with(Transform::default)
                .set_scale(Vector3::new(scale, scale, scale));
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

impl Into<BoundingSphere> for GltfNodeExtent {
    fn into(self) -> BoundingSphere {
        BoundingSphere {
            center: convert(self.centroid()),
            radius: convert(self.distance().magnitude() * 0.5),
        }
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

/// Used during gltf loading to contain the materials used from scenes in the file
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct GltfMaterialSet {
    pub(crate) materials: HashMap<usize, MaterialPrefab>,
}

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(default)]
pub struct GltfSceneOptions {
    /// Generate texture coordinates if none exist in the Gltf file
    pub generate_tex_coords: (f32, f32),
    #[derivative(Default(value = "true"))]
    /// Load vertex normal data from the Gltf file
    pub load_normals: bool,
    #[derivative(Default(value = "true"))]
    /// Load vertex color data from the Gltf file
    pub load_colors: bool,
    #[derivative(Default(value = "true"))]
    /// Load texture coordinates data from the Gltf file
    pub load_texcoords: bool,
    #[derivative(Default(value = "true"))]
    /// Load vertex tangent data from the Gltf file
    pub load_tangents: bool,
    #[derivative(Default(value = "true"))]
    /// Load animation data from the Gltf file
    pub load_animations: bool,
    /// Flip the v coordinate for all texture coordinates
    pub flip_v_coord: bool,
    /// Load the given scene index, if not supplied will either load the default scene (if set),
    /// or the first scene (only if there is only one scene, otherwise an `Error` will be returned).
    pub scene_index: Option<usize>,
}

type SysDataOf<'a, T> = <T as PrefabData<'a>>::SystemData;

impl<'a> PrefabData<'a> for GltfPrefab {
    // Looks better when defined all in one place
    #[allow(clippy::type_complexity)]
    type SystemData = (
        SysDataOf<'a, Transform>,
        SysDataOf<'a, Named>,
        SysDataOf<'a, CameraPrefab>,
        SysDataOf<'a, LightPrefab>,
        SysDataOf<'a, MaterialPrefab>,
        SysDataOf<'a, AnimatablePrefab<usize, Transform>>,
        SysDataOf<'a, SkinnablePrefab>,
        WriteStorage<'a, BoundingSphere>,
        WriteStorage<'a, Handle<Mesh>>,
        Read<'a, AssetStorage<Mesh>>,
        ReadExpect<'a, Loader>,
        Write<'a, GltfMaterialSet>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
        children: &[Entity],
    ) -> Result<(), Error> {
        let (
            transforms,
            names,
            cameras,
            lights,
            materials,
            animatables,
            skinnables,
            bound,
            meshes,
            _,
            _,
            _,
        ) = system_data;
        if let Some(transform) = &self.transform {
            transform.add_to_entity(entity, transforms, entities, children)?;
        }
        if let Some(mesh) = &self.mesh_handle {
            meshes.insert(entity, mesh.clone())?;
        }
        if let Some(camera) = &self.camera {
            camera.add_to_entity(entity, cameras, entities, children)?;
        }
        if let Some(light) = &self.light {
            light.add_to_entity(entity, lights, entities, children)?;
        }
        if let Some(name) = &self.name {
            name.add_to_entity(entity, names, entities, children)?;
        }
        if let Some(material) = &self.material {
            material.add_to_entity(entity, materials, entities, children)?;
        }
        if let Some(animatable) = &self.animatable {
            animatable.add_to_entity(entity, animatables, entities, children)?;
        }
        if let Some(skinnable) = &self.skinnable {
            skinnable.add_to_entity(entity, skinnables, entities, children)?;
        }
        if let Some(extent) = &self.extent {
            bound.insert(entity, extent.clone().into())?;
        }
        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        system_data: &mut Self::SystemData,
    ) -> Result<bool, Error> {
        let (_, _, _, _, materials, animatables, _, _, _, meshes_storage, loader, mat_set) =
            system_data;

        let mut ret = false;
        if let Some(mut mats) = self.materials.take() {
            mat_set.materials.clear();
            for (id, mut material) in mats.materials.drain() {
                ret |= material.load_sub_assets(progress, materials)?;
                mat_set.materials.insert(id, material);
            }
        }
        if let Some(material_id) = self.material_id {
            if let Some(mat) = mat_set.materials.get(&material_id) {
                self.material.replace(mat.clone_loaded());
            }
        }
        if let Some(mesh) = self.mesh.take() {
            self.mesh_handle =
                Some(loader.load_from_data(mesh.clone().into(), &mut *progress, meshes_storage));
            ret = true;
        }
        if let Some(animatable) = &mut self.animatable {
            ret |= animatable.load_sub_assets(progress, animatables)?;
        }
        Ok(ret)
    }
}
