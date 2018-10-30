//! Physically-based material.

use fnv::FnvHashMap;

use amethyst_assets::Handle;
use amethyst_core::specs::prelude::{Component, DenseVecStorage};

use tex::{Texture, TextureHandle};

/// Material reference this part of the texture
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TextureOffset {
    /// Start and end offset for U coordinate
    pub u: (f32, f32),
    /// Start and end offset for V coordinate
    pub v: (f32, f32),
}

impl Default for TextureOffset {
    fn default() -> Self {
        TextureOffset {
            u: (0., 1.),
            v: (0., 1.),
        }
    }
}

/// Material struct.
#[derive(Clone, PartialEq)]
pub struct Material {
    /// Alpha cutoff: the value at which we do not draw the pixel
    pub alpha_cutoff: f32,
    /// Diffuse map.
    pub albedo: TextureHandle,
    /// Diffuse texture offset
    pub albedo_offset: TextureOffset,
    /// Emission map.
    pub emission: TextureHandle,
    /// Emission texture offset
    pub emission_offset: TextureOffset,
    /// Normal map.
    pub normal: TextureHandle,
    /// Normal texture offset
    pub normal_offset: TextureOffset,
    /// Metallic map.
    pub metallic: TextureHandle,
    /// Metallic texture offset
    pub metallic_offset: TextureOffset,
    /// Roughness map.
    pub roughness: TextureHandle,
    /// Roughness texture offset
    pub roughness_offset: TextureOffset,
    /// Ambient occlusion map.
    pub ambient_occlusion: TextureHandle,
    /// Ambient occlusion texture offset
    pub ambient_occlusion_offset: TextureOffset,
    /// Caveat map.
    pub caveat: TextureHandle,
    /// Caveat texture offset
    pub caveat_offset: TextureOffset,
}

impl Component for Material {
    type Storage = DenseVecStorage<Self>;
}

/// A resource providing default textures for `Material`.
/// These will be be used by the renderer in case a texture
/// handle points to a texture which is not loaded already.
/// Additionally, you can use it to fill up the fields of
/// `Material` you don't want to specify.
#[derive(Clone)]
pub struct MaterialDefaults(pub Material);

/// Textures used by texture animations
#[derive(Debug, Default)]
pub struct MaterialTextureSet {
    textures: FnvHashMap<u64, Handle<Texture>>,
    texture_inverse: FnvHashMap<Handle<Texture>, u64>,
}

impl MaterialTextureSet {
    /// Create new texture set
    pub fn new() -> Self {
        MaterialTextureSet {
            textures: FnvHashMap::default(),
            texture_inverse: FnvHashMap::default(),
        }
    }

    /// Retrieve the handle for a given index
    pub fn handle(&self, id: u64) -> Option<Handle<Texture>> {
        self.textures.get(&id).cloned()
    }

    /// Retrieve the index for a given handle
    pub fn id(&self, handle: &Handle<Texture>) -> Option<u64> {
        self.texture_inverse.get(handle).cloned()
    }

    /// Insert a texture handle at the given index
    pub fn insert(&mut self, id: u64, handle: Handle<Texture>) {
        self.textures.insert(id, handle.clone());
        self.texture_inverse.insert(handle, id);
    }

    /// Remove the given index
    pub fn remove(&mut self, id: u64) {
        if let Some(handle) = self.textures.remove(&id) {
            self.texture_inverse.remove(&handle);
        }
    }

    /// Get number of textures in the set
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Remove all texture handles in the set
    pub fn clear(&mut self) {
        self.textures.clear();
        self.texture_inverse.clear();
    }
}
