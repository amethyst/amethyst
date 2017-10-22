extern crate amethyst_assets as assets;
extern crate amethyst_core as core;
extern crate amethyst_renderer as renderer;
extern crate base64;
extern crate gfx;
extern crate gltf;
extern crate gltf_utils;
extern crate imagefmt;
extern crate itertools;
extern crate specs;

pub use format::GltfSceneFormat;
pub use systems::GltfSceneLoaderSystem;

use assets::{Asset, BoxedErr, Handle};
use core::transform::LocalTransform;
use gfx::Primitive;
use renderer::{MeshHandle, TextureData, TextureHandle, VertexBufferCombination};
use specs::DenseVecStorage;

mod format;
mod systems;

/// A single graphics primitive
#[derive(Debug)]
pub struct GltfPrimitive {
    pub primitive: Primitive,
    pub material: Option<usize>,
    pub indices: Option<Vec<usize>>,
    pub attributes: VertexBufferCombination,
    pub handle: Option<MeshHandle>,
}

/// Alpha mode for material
#[derive(Debug)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

/// GLTF material, PBR based
#[derive(Debug)]
pub struct GltfMaterial {
    base_color: (GltfTexture, [f32; 4]),
    metallic: (GltfTexture, f32),
    roughness: (GltfTexture, f32),
    emissive: (GltfTexture, [f32; 3]),
    normal: Option<(GltfTexture, f32)>,
    occlusion: Option<(GltfTexture, f32)>,
    alpha: (AlphaMode, f32),
    double_sided: bool,
}

/// A GLTF defined texture, will be in `TextureData` format in the output from the loader.
#[derive(Debug)]
pub struct GltfTexture {
    pub data: TextureData,
    pub handle: Option<TextureHandle>,
}

impl GltfTexture {
    pub fn new(data: TextureData) -> Self {
        Self { data, handle: None }
    }
}

/// A node in the scene hierarchy
#[derive(Debug)]
pub struct GltfNode {
    pub primitives: Vec<GltfPrimitive>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub local_transform: LocalTransform,
}

/// A single scene is defined as a list of the root nodes in the node hierarchy for the full asset
#[derive(Debug)]
pub struct GltfScene {
    pub root_nodes: Vec<usize>,
}

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Default)]
pub struct GltfSceneOptions {
    pub generate_tex_coords: Option<(f32, f32)>,
}

/// Actual asset produced on finished loading of a GLTF scene file.
#[derive(Debug)]
pub struct GltfSceneAsset {
    pub nodes: Vec<GltfNode>,
    pub scenes: Vec<GltfScene>,
    pub materials: Vec<GltfMaterial>,
    pub default_scene: Option<usize>,
    pub options: GltfSceneOptions,
}

impl Into<Result<GltfSceneAsset, BoxedErr>> for GltfSceneAsset {
    fn into(self) -> Result<GltfSceneAsset, BoxedErr> {
        Ok(self)
    }
}

impl Asset for GltfSceneAsset {
    type Data = Self;
    // TODO: replace by tracked storage
    type HandleStorage = DenseVecStorage<Handle<Self>>;
}
