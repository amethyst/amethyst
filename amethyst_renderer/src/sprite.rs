use amethyst_assets::{
    Asset, AssetStorage, Handle, Loader, ProcessingState, Result as AssetsResult,
};
use amethyst_core::specs::prelude::{
    Component, Entity, EntityBuilder, Read, ReadExpect, VecStorage, WriteStorage,
};
use error::Result;
use fnv::FnvHashMap;
use mesh::{Mesh, MeshHandle};
use shape::Shape;
use std::marker::Sized;
use tex::TextureHandle;
use {Material, MaterialDefaults, PosTex, TextureOffset};

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetHandle = Handle<SpriteSheet>;

/// Meta data for a sprite sheet texture.
///
/// Contains a handle to the texture and the sprite coordinates on the texture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheet {
    /// Index into `MaterialTextureSet` of the texture for this sprite sheet.
    pub texture_id: u64,
    /// A list of sprites in this sprite sheet.
    pub sprites: Vec<Sprite>,
}

impl Asset for SpriteSheet {
    const NAME: &'static str = "renderer::SpriteSheet";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl From<SpriteSheet> for AssetsResult<ProcessingState<SpriteSheet>> {
    fn from(sprite_sheet: SpriteSheet) -> AssetsResult<ProcessingState<SpriteSheet>> {
        Ok(ProcessingState::Loaded(sprite_sheet))
    }
}

/// Dimensions and texture coordinates of each sprite in a sprite sheet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprite {
    /// Pixel width of the sprite
    pub width: f32,
    /// Pixel height of the sprite
    pub height: f32,
    /// Number of pixels to shift the sprite to the left and down relative to the entity
    pub offsets: [f32; 2],
    /// Texture coordinates of the sprite
    pub tex_coords: TextureCoordinates,
}

/// Texture coordinates of the sprite
///
/// The coordinates should be normalized to a value between 0.0 and 1.0:
///
/// * X axis: 0.0 is the left side and 1.0 is the right side.
/// * Y axis: 0.0 is the top and 1.0 is the bottom.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureCoordinates {
    /// Normalized left x coordinate
    pub left: f32,
    /// Normalized right x coordinate
    pub right: f32,
    /// Normalized top y coordinate
    pub top: f32,
    /// Normalized bottom y coordinate
    pub bottom: f32,
}

impl From<((f32, f32), [f32; 4])> for Sprite {
    fn from((dimensions, tex_coords): ((f32, f32), [f32; 4])) -> Self {
        Self::from((dimensions, [0.0; 2], tex_coords))
    }
}

impl From<((f32, f32), [f32; 2], [f32; 4])> for Sprite {
    fn from(((width, height), offsets, tex_coords): ((f32, f32), [f32; 2], [f32; 4])) -> Self {
        Sprite {
            width,
            height,
            offsets,
            tex_coords: TextureCoordinates::from(tex_coords),
        }
    }
}

impl From<((f32, f32), (f32, f32))> for TextureCoordinates {
    fn from(((left, right), (top, bottom)): ((f32, f32), (f32, f32))) -> Self {
        TextureCoordinates {
            left,
            right,
            top,
            bottom,
        }
    }
}

impl From<[f32; 4]> for TextureCoordinates {
    fn from(uv: [f32; 4]) -> Self {
        TextureCoordinates {
            left: uv[0],
            right: uv[1],
            top: uv[2],
            bottom: uv[3],
        }
    }
}

/// Information for rendering a sprite.
///
/// Instead of using a `Mesh` on a `DrawFlat` render pass, we can use a simpler set of shaders to
/// render sprites. This struct carries the information necessary for the sprite pass.
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteRender {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: SpriteSheetHandle,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
    /// Whether the sprite should be flipped horizontally
    pub flip_horizontal: bool,
    /// Whether the sprite should be flipped horizontally
    pub flip_vertical: bool,
}

impl Component for SpriteRender {
    type Storage = VecStorage<Self>;
}

/// SystemData containing the data necessary to handle new rendered sprites
#[derive(SystemData)]
pub struct SpriteRenderData<'a> {
    /// Storage containing the meshes
    pub meshes: WriteStorage<'a, MeshHandle>,
    /// Storage containing the materials
    pub materials: WriteStorage<'a, Material>,
    /// Material defaults
    pub material_defaults: ReadExpect<'a, MaterialDefaults>,
    /// Asset loader
    pub loader: ReadExpect<'a, Loader>,
    /// Mesh asset storage
    pub asset_storage: Read<'a, AssetStorage<Mesh>>,
}

impl<'a> SpriteRenderData<'a> {
    /// Creates a MeshHandle and Material from the sprite and texture data.
    /// Useful if you plan on re-using the same sprite a lot and don't want to
    /// load the assets each time.
    pub fn build_mesh_and_material(
        &mut self,
        sprite: &Sprite,
        texture: TextureHandle,
        size: (f32, f32),
    ) -> (MeshHandle, Material) {
        let vertices = Shape::Plane(None).generate::<Vec<PosTex>>(Some((
            sprite.offsets[0],
            sprite.offsets[1],
            0.0,
        )));
        let mesh = self
            .loader
            .load_from_data(vertices, (), &self.asset_storage);

        let material = Material {
            albedo: texture,
            albedo_offset: TextureOffset {
                u: (
                    sprite.tex_coords.left / size.0,
                    sprite.tex_coords.right / size.0,
                ),
                v: (
                    1.0 - sprite.tex_coords.bottom / size.1,
                    1.0 - sprite.tex_coords.top / size.1,
                ),
            },
            ..self.material_defaults.0.clone()
        };

        (mesh, material)
    }

    /// Adds a mesh and a material to an entity corresponding to the sprite and texture given.
    /// Note that is you need to insert the same sprite and texture, using ``add_multiple`` allows for better performances.
    pub fn add(
        &mut self,
        entity: Entity,
        sprite: &Sprite,
        texture: TextureHandle,
        texture_size: (f32, f32),
    ) -> Result<()> {
        let (mesh, material) = self.build_mesh_and_material(sprite, texture, texture_size);
        self.meshes.insert(entity, mesh)?;
        self.materials.insert(entity, material)?;
        Ok(())
    }

    /// Adds the same mesh and material to multiple entities corresponding to the sprite and texture given.
    pub fn add_multiple(
        &mut self,
        entities: Vec<Entity>,
        sprite: &Sprite,
        texture: TextureHandle,
        texture_size: (f32, f32),
    ) -> Result<()> {
        let len = entities.len();
        if len != 0 {
            let (mesh, material) = self.build_mesh_and_material(sprite, texture, texture_size);
            for entity in 0..len - 1 {
                self.meshes.insert(entities[entity], mesh.clone())?;
                self.materials.insert(entities[entity], material.clone())?;
            }
            self.meshes.insert(entities[len - 1], mesh)?;
            self.materials.insert(entities[len - 1], material)?;
        }
        Ok(())
    }
}

/// An easy way to attach and display a sprite when building an entity
pub trait WithSpriteRender
where
    Self: Sized,
{
    /// Adds a mesh and a material to the entity being built corresponding to the sprite and texture given.
    fn with_sprite(
        self,
        sprite: &Sprite,
        texture: TextureHandle,
        texture_size: (f32, f32),
    ) -> Result<Self>;
}

impl<'a> WithSpriteRender for EntityBuilder<'a> {
    fn with_sprite(
        self,
        sprite: &Sprite,
        texture: TextureHandle,
        texture_size: (f32, f32),
    ) -> Result<Self> {
        self.world.system_data::<SpriteRenderData>().add(
            self.entity,
            sprite,
            texture,
            texture_size,
        )?;
        Ok(self)
    }
}

/// Sprite sheets used by sprite render animations
///
/// In sprite animations, it is plausible to switch the `SpriteSheet` during the animation.
/// `Animation`s require their primitives to be `Copy`. However, `Handle<SpriteSheet>`s are `Clone`
/// but not `Copy`. Therefore, to allow switching of the `SpriteSheet`, we use a `Copy` ID, and map
/// that to the sprite sheet handle so that it can be looked up when being sampled in the animation.
#[derive(Debug, Default)]
pub struct SpriteSheetSet {
    sprite_sheets: FnvHashMap<u64, SpriteSheetHandle>,
    sprite_sheet_inverse: FnvHashMap<SpriteSheetHandle, u64>,
}

impl SpriteSheetSet {
    /// Create new sprite sheet set
    pub fn new() -> Self {
        SpriteSheetSet {
            sprite_sheets: FnvHashMap::default(),
            sprite_sheet_inverse: FnvHashMap::default(),
        }
    }

    /// Retrieve the handle for a given index
    pub fn handle(&self, id: u64) -> Option<SpriteSheetHandle> {
        self.sprite_sheets.get(&id).cloned()
    }

    /// Retrieve the index for a given handle
    pub fn id(&self, handle: &SpriteSheetHandle) -> Option<u64> {
        self.sprite_sheet_inverse.get(handle).cloned()
    }

    /// Insert a sprite sheet handle at the given index
    pub fn insert(&mut self, id: u64, handle: SpriteSheetHandle) {
        self.sprite_sheets.insert(id, handle.clone());
        self.sprite_sheet_inverse.insert(handle, id);
    }

    /// Remove the given index
    pub fn remove(&mut self, id: u64) {
        if let Some(handle) = self.sprite_sheets.remove(&id) {
            self.sprite_sheet_inverse.remove(&handle);
        }
    }

    /// Get number of sprite sheets in the set
    pub fn len(&self) -> usize {
        self.sprite_sheets.len()
    }

    /// Returns whether the set contains any sprite sheets
    pub fn is_empty(&self) -> bool {
        self.sprite_sheets.is_empty()
    }

    /// Remove all sprite sheet handles in the set
    pub fn clear(&mut self) {
        self.sprite_sheets.clear();
        self.sprite_sheet_inverse.clear();
    }
}

#[cfg(test)]
mod test {
    use super::{Sprite, TextureCoordinates};

    #[test]
    fn texture_coordinates_from_tuple_maps_fields_correctly() {
        assert_eq!(
            TextureCoordinates {
                left: 0.,
                right: 0.5,
                top: 0.75,
                bottom: 1.0,
            },
            ((0.0, 0.5), (0.75, 1.0)).into()
        );
    }

    #[test]
    fn texture_coordinates_from_slice_maps_fields_correctly() {
        assert_eq!(
            TextureCoordinates {
                left: 0.,
                right: 0.5,
                top: 0.75,
                bottom: 1.0,
            },
            [0.0, 0.5, 0.75, 1.0].into()
        );
    }

    #[test]
    fn sprite_from_tuple_maps_fields_correctly() {
        assert_eq!(
            Sprite {
                width: 10.,
                height: 40.,
                offsets: [5., 20.],
                tex_coords: TextureCoordinates {
                    left: 0.,
                    right: 0.5,
                    top: 0.75,
                    bottom: 1.0,
                },
            },
            ((10., 40.), [5., 20.], [0.0, 0.5, 0.75, 1.0]).into()
        );
    }

    #[test]
    fn sprite_offsets_default_to_zero() {
        assert_eq!(
            Sprite {
                width: 10.,
                height: 40.,
                offsets: [0., 0.],
                tex_coords: TextureCoordinates {
                    left: 0.,
                    right: 0.5,
                    top: 0.75,
                    bottom: 1.0,
                },
            },
            ((10., 40.), [0.0, 0.5, 0.75, 1.0]).into()
        );
    }
}
