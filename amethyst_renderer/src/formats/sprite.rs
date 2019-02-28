use ron::de::from_bytes as from_ron_bytes;
use serde::{Deserialize, Serialize};

use amethyst_assets::{AssetStorage, Handle, Loader, PrefabData, ProgressCounter, SimpleFormat};
use amethyst_core::ecs::prelude::{Entity, Read, ReadExpect, WriteStorage};
use amethyst_error::Error;

use crate::{error, Sprite, SpriteRender, SpriteSheet, Texture, TextureFormat, TexturePrefab};

/// Structure acting as scaffolding for serde when loading a spritesheet file.
/// Positions originate in the top-left corner (bitmap image convention).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpritePosition {
    /// Horizontal position of the sprite in the sprite sheet
    pub x: u32,
    /// Vertical position of the sprite in the sprite sheet
    pub y: u32,
    /// Width of the sprite
    pub width: u32,
    /// Height of the sprite
    pub height: u32,
    /// Number of pixels to shift the sprite to the left and down relative to the entity holding it
    pub offsets: Option<[f32; 2]>,
}

/// Structure acting as scaffolding for serde when loading a spritesheet file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerializedSpriteSheet {
    /// Width of the sprite sheet
    pub spritesheet_width: u32,
    /// Height of the sprite sheet
    pub spritesheet_height: u32,
    /// Description of the sprites
    pub sprites: Vec<SpritePosition>,
}

/// Allows loading of sprite sheets in RON format.
///
/// This format allows to conveniently load a sprite sheet from a RON file.
///
/// Example:
/// ```text,ignore
/// (
///     // Width of the sprite sheet
///     spritesheet_width: 48.0,
///     // Height of the sprite sheet
///     spritesheet_height: 16.0,
///     // List of sprites the sheet holds
///     sprites: [
///         (
///             // Horizontal position of the sprite in the sprite sheet
///             x: 0.0,
///             // Vertical position of the sprite in the sprite sheet
///             y: 0.0,
///             // Width of the sprite
///             width: 16.0,
///             // Height of the sprite
///             height: 16.0,
///             // Number of pixels to shift the sprite to the left and down relative to the entity holding it when rendering
///             offsets: (0.0, 0.0), // This is optional and defaults to (0.0, 0.0)
///         ),
///         (
///             x: 16.0,
///             y: 0.0,
///             width: 32.0,
///             height: 16.0,
///         ),
///     ],
/// )
/// ```
///
/// Such a spritesheet description can be loaded using a `Loader` by passing it the handle of the corresponding loaded texture.
/// ```rust,no_run
/// # use amethyst_assets::{Loader, AssetStorage};
/// # use amethyst_renderer::{SpriteSheetFormat, SpriteSheet, Texture, PngFormat, TextureMetadata};
/// #
/// # fn load_sprite_sheet() {
/// #   let world = amethyst_core::specs::World::new(); // Normally, you would use Amethyst's world
/// #   let loader = world.read_resource::<Loader>();
/// #   let spritesheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
/// #   let texture_storage = world.read_resource::<AssetStorage<Texture>>();
/// let texture_handle = loader.load(
///     "my_texture.png",
///     PngFormat,
///     TextureMetadata::srgb(),
///     (),
///     &texture_storage,
/// );
/// let spritesheet_handle = loader.load(
///     "my_spritesheet.ron",
///     SpriteSheetFormat,
///     texture_handle,
///     (),
///     &spritesheet_storage,
/// );
/// # }
/// ```
#[derive(Clone, Deserialize, Serialize)]
pub struct SpriteSheetFormat;

impl SimpleFormat<SpriteSheet> for SpriteSheetFormat {
    const NAME: &'static str = "SPRITE_SHEET";

    type Options = Handle<Texture>;

    fn import(&self, bytes: Vec<u8>, texture: Self::Options) -> Result<SpriteSheet, Error> {
        let sheet: SerializedSpriteSheet =
            from_ron_bytes(&bytes).map_err(|_| error::Error::LoadSpritesheetError)?;

        let mut sprites: Vec<Sprite> = Vec::with_capacity(sheet.sprites.len());
        for sp in sheet.sprites {
            let sprite = Sprite::from_pixel_values(
                sheet.spritesheet_width as u32,
                sheet.spritesheet_height as u32,
                sp.width as u32,
                sp.height as u32,
                sp.x as u32,
                sp.y as u32,
                sp.offsets.unwrap_or([0.0; 2]),
            );
            sprites.push(sprite);
        }
        Ok(SpriteSheet { texture, sprites })
    }
}

/// `PrefabData` for loading `SpriteRender`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteRenderPrefab {
    /// Spritesheet texture
    pub texture: TexturePrefab<TextureFormat>,
    /// Sprite coordinates on the texture
    pub sprite_sheet: SerializedSpriteSheet,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl<'a> PrefabData<'a> for SpriteRenderPrefab {
    type SystemData = (
        <TexturePrefab<TextureFormat> as PrefabData<'a>>::SystemData,
        ReadExpect<'a, Loader>,
        Read<'a, AssetStorage<SpriteSheet>>,
        WriteStorage<'a, SpriteRender>,
    );
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        entities: &[Entity],
    ) -> Result<(), Error> {
        let (tex_data, loader, sheet_storage, render_storage) = system_data;

        let mut sprites: Vec<Sprite> = Vec::with_capacity(self.sprite_sheet.sprites.len());
        for sp in &self.sprite_sheet.sprites {
            let sprite = Sprite::from_pixel_values(
                self.sprite_sheet.spritesheet_width as u32,
                self.sprite_sheet.spritesheet_height as u32,
                sp.width as u32,
                sp.height as u32,
                sp.x as u32,
                sp.y as u32,
                sp.offsets.unwrap_or([0.0; 2]),
            );
            sprites.push(sprite);
        }

        let texture = self.texture.add_to_entity(entity, tex_data, entities)?;

        let sheet = SpriteSheet { texture, sprites };
        let sheet_handle = loader.load_from_data(sheet, (), sheet_storage);

        let render = SpriteRender {
            sprite_sheet: sheet_handle,
            sprite_number: self.sprite_number,
        };
        render_storage.insert(entity, render)?;

        Ok(())
    }

    fn load_sub_assets(
        &mut self,
        progress: &mut ProgressCounter,
        (tex_data, _, _, _): &mut Self::SystemData,
    ) -> Result<bool, Error> {
        self.texture.load_sub_assets(progress, tex_data)
    }
}
