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
extern crate specs;

pub use format::GltfSceneFormat;
pub use systems::GltfSceneLoaderSystem;

use std::ops::Range;

use animation::{Animation, Sampler};
use assets::{Asset, Error as AssetError, Handle};
use core::transform::LocalTransform;
use gfx::Primitive;
use renderer::{AnimatedVertexBufferCombination, MeshHandle, TextureData, TextureHandle};
use specs::VecStorage;

mod format;
mod systems;

/// A single graphics primitive
#[derive(Debug)]
pub struct GltfPrimitive {
    pub extents: Range<[f32; 3]>,
    pub primitive: Primitive,
    pub material: Option<usize>,
    pub indices: Option<Vec<usize>>,
    pub attributes: AnimatedVertexBufferCombination,
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

/// A GLTF defined skin
#[derive(Debug)]
pub struct GltfSkin {
    pub joints: Vec<usize>,
    pub skeleton: Option<usize>,
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}

/// A node in the scene hierarchy
#[derive(Debug)]
pub struct GltfNode {
    pub primitives: Vec<GltfPrimitive>,
    pub skin: Option<usize>,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub local_transform: LocalTransform,
}

/// A single scene is defined as a list of the root nodes in the node hierarchy for the full asset
#[derive(Debug)]
pub struct GltfScene {
    pub root_nodes: Vec<usize>,
}

/// A single animation
#[derive(Debug)]
pub struct GltfAnimation {
    // node index, vec will be same size as samplers, and reference the sampler at the same index
    pub nodes: Vec<usize>,
    pub samplers: Vec<Sampler>,
    pub handle: Option<Handle<Animation>>,
    //pub hierarchy_root: usize,
}

/// Options used when loading a GLTF file
#[derive(Debug, Clone, Default)]
pub struct GltfSceneOptions {
    pub generate_tex_coords: Option<(f32, f32)>,
    pub load_animations: bool,
    pub flip_v_coord: bool,
    pub move_to_origin: bool,
}

/// Actual asset produced on finished loading of a GLTF scene file.
#[derive(Debug)]
pub struct GltfSceneAsset {
    pub nodes: Vec<GltfNode>,
    pub scenes: Vec<GltfScene>,
    pub materials: Vec<GltfMaterial>,
    pub animations: Vec<GltfAnimation>,
    pub default_scene: Option<usize>,
    pub skins: Vec<GltfSkin>,
    pub options: GltfSceneOptions,
}

impl Into<Result<GltfSceneAsset, AssetError>> for GltfSceneAsset {
    fn into(self) -> Result<GltfSceneAsset, AssetError> {
        Ok(self)
    }
}

impl Asset for GltfSceneAsset {
    const NAME: &'static str = "gltf::Scene";
    type Data = Self;
    // TODO: replace by tracked storage
    type HandleStorage = VecStorage<Handle<Self>>;
}
