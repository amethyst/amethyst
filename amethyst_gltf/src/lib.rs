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

use amethyst_assets::{AssetStorage, Handle, Loader, ProgressCounter};
use amethyst_core::{
    ecs::prelude::{Entity, Read, ReadExpect, Write, WriteStorage},
    math::{convert, Point3, Vector3},
    transform::Transform,
    Named,
};
use amethyst_error::Error;
use amethyst_rendy::{rendy::mesh::MeshBuilder, types::Mesh, visibility::BoundingSphere};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

pub use crate::format::GltfSceneFormat;

mod error;
mod format;

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
