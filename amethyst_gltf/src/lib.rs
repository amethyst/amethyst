//! A crate for loading GLTF format scenes into Amethyst

#![doc(
html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(
missing_debug_implementations,
missing_docs,
rust_2018_idioms,
rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

use std::{collections::HashMap, ops::Range};

use amethyst_assets::{AssetStorage, Handle, Loader, ProgressCounter, prefab::Prefab, inventory, Asset, AssetProcessorSystem};
use amethyst_core::{
    ecs::{Entity, Read, ReadExpect, Write, WriteStorage},
    ecs::*,
    math::{convert, Point3, Vector3},
    transform::Transform,
    Named,
};
use amethyst_error::Error;
use amethyst_rendy::{rendy::mesh::MeshBuilder, types::Mesh, visibility::BoundingSphere, Camera, Material};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use amethyst_rendy::light::Light;
use amethyst_animation::Skin;

mod error;
mod importer;

pub use importer::GltfImporter;

inventory::submit!{
    amethyst_assets::SourceFileImporter {
        extension: "gltf",
        instantiator: || Box::new(GltfImporter::default()),
    }
}

inventory::submit!{
    amethyst_assets::SourceFileImporter {
        extension: "glb",
        instantiator: || Box::new(GltfImporter::default()),
    }
}

/// A GLTF node extent
#[derive(Clone, Debug, Serialize)]
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
#[derive(Derivative, Serialize)]
#[derivative(Default(bound = ""))]
pub struct GltfMaterialSet {
    pub(crate) materials: HashMap<usize, Prefab>,
}

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Default, Derivative, Serialize, Deserialize, TypeUuid)]
#[serde(default)]
#[uuid = "8e3da51a-26d4-4b0f-b9f7-7f52d1b78945"]
pub struct GltfSceneOptions {
    /// Path of the gltf scene file
    pub scene_path: String,
    /// Name of the current scene
    pub scene_name: String,
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

/// `AssetData` for gltf objects.
#[derive(Default, TypeUuid, Serialize)]
#[uuid = "8a7b7733-d770-4400-8ea8-b82dbc10aae2"]
pub struct GltfAsset {
    /// `Transform` will almost always be placed, the only exception is for the main `Entity` for
    /// certain scenarios (based on the data in the Gltf file)
    pub transform: Option<Transform>,
    /// `Camera` will always be placed
    pub camera: Option<Camera>,
    /// Lights can be added to a prefab with the `KHR_lights_punctual` feature enabled
    pub light: Option<Light>,
    /// `MeshData` is placed on all `Entity`s with graphics primitives
    pub mesh: Option<MeshBuilder<'static>>,
    /// Mesh handle after sub asset loading is done
    pub mesh_handle: Option<Handle<Mesh>>,
    /// `Material` is placed on all `Entity`s with graphics primitives with material
    pub material: Option<Material>,
    /// Loaded animations, if applicable, will always only be placed on the main `Entity`
  // pub animatable: Option<Animatable<usize, Transform>>,
    /// Skin data is placed on `Entity`s involved in the skin, skeleton or graphical primitives
    /// using the skin
    pub skin: Option<Skin>,
    /// Node extent
    pub extent: Option<GltfNodeExtent>,
    /// Node name
    pub name: Option<Named>,
    /// Node index when loading a full scene
    pub index: usize,
    pub(crate) materials: Option<GltfMaterialSet>,
    pub(crate) material_id: Option<usize>,
}

impl Asset for GltfAsset {
    fn name() -> &'static str {
        "Texture"
    }
    type Data = ();
}