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

use amethyst_animation::{Animation, Joint};
use amethyst_assets::{
    inventory, prefab::register_component_type, register_asset_type, AssetProcessorSystem,
};
use amethyst_core::Transform;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

/// Bundle that initializes needed resources to use GLTF
pub mod bundle;
mod importer;
mod system;
mod types;

pub use importer::GltfImporter;

inventory::submit! {
    amethyst_assets::SourceFileImporter {
        extension: "gltf",
        instantiator: || Box::new(GltfImporter::default()),
    }
}

inventory::submit! {
    amethyst_assets::SourceFileImporter {
        extension: "glb",
        instantiator: || Box::new(GltfImporter::default()),
    }
}

register_component_type!(Joint);
register_asset_type!(Animation<Transform> => Animation<Transform>; AssetProcessorSystem<Animation<Transform>>);

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Derivative, Serialize, Deserialize, TypeUuid)]
#[serde(default)]
#[derivative(Default)]
#[uuid = "8e3da51a-26d4-4b0f-b9f7-7f52d1b78945"]
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
