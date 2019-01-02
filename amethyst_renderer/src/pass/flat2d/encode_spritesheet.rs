use super::Flat2DData;
use crate::{Flipped, Hidden, HiddenPropagate, Rgba, SpriteSheet, Transparent};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    specs::{Component, Join, Read, ReadStorage, System, VecStorage, Write},
    GlobalTransform,
};

use super::encode_sprite::encode_sprite;

/// A component that guides `DrawFlat2D` pass encoding for rendering a sprite in a spritesheet.
#[derive(Clone, Debug)]
pub struct RenderSpriteSheetFlat2D {
    /// Handle to the sprite sheet of the sprite
    pub sprite_sheet: Handle<SpriteSheet>,
    /// Index of the sprite on the sprite sheet
    pub sprite_number: usize,
}

impl Component for RenderSpriteSheetFlat2D {
    type Storage = VecStorage<Self>;
}

/// An encoder system that prepares entities with `RenderSpriteSheetFlat2D` component
/// to be drawn using `DrawFlat2D` render pass.
#[derive(Clone, Debug, Default)]
pub struct Flat2DSpriteSheetEncoder;
impl<'a> System<'a> for Flat2DSpriteSheetEncoder {
    type SystemData = (
        Write<'a, Vec<Flat2DData>>,
        ReadStorage<'a, RenderSpriteSheetFlat2D>,
        Read<'a, AssetStorage<SpriteSheet>>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Flipped>,
        ReadStorage<'a, Rgba>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
    );
    fn run(
        &mut self,
        (mut buffer, renders, storage, transforms, flips, rgbas, transparent, hidden, hidden_prop): Self::SystemData,
    ) {
        for (render, transform, flip, rgba, transparent, _, _) in (
            &renders,
            &transforms,
            flips.maybe(),
            rgbas.maybe(),
            transparent.maybe(),
            !&hidden,
            !&hidden_prop,
        )
            .join()
        {
            if let Some(sprite_sheet) = storage.get(&render.sprite_sheet) {
                encode_sprite(
                    &mut buffer,
                    sprite_sheet.texture.clone(),
                    &sprite_sheet.frames[render.sprite_number],
                    &transform,
                    flip,
                    rgba.cloned().unwrap_or(Rgba::WHITE),
                    transparent.is_some(),
                );
            }
        }
    }
}
