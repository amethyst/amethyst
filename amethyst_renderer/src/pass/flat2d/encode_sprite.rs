use super::Flat2DData;
use crate::{Flipped, Hidden, HiddenPropagate, Rgba, Sprite, SpriteFrame, Texture, Transparent};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    nalgebra::Vector4,
    specs::{Component, Join, Read, ReadStorage, System, VecStorage, Write},
    GlobalTransform,
};

/// Information for rendering a standalone sprite.
///
/// Instead of using a `Mesh` on a `DrawFlat` render pass, we can use a simpler set of shaders to
/// render textures to quads. This struct carries the information necessary for the `DrawFlat2D` pass.
#[derive(Clone, Debug)]
pub struct RenderSpriteFlat2D(pub Handle<Sprite>);

impl Component for RenderSpriteFlat2D {
    type Storage = VecStorage<Self>;
}

/// An encoder system that prepares entities with `RenderSpriteFlat2D` component
/// to be drawn using `DrawFlat2D` render pass.
#[derive(Clone, Debug, Default)]
pub struct Flat2DSpriteEncoder;
impl<'a> System<'a> for Flat2DSpriteEncoder {
    type SystemData = (
        Write<'a, Vec<Flat2DData>>,
        ReadStorage<'a, RenderSpriteFlat2D>,
        Read<'a, AssetStorage<Sprite>>,
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
            if let Some(sprite) = storage.get(&render.0) {
                encode_sprite(
                    &mut buffer,
                    sprite.texture.clone(),
                    &sprite.frame,
                    &transform,
                    flip,
                    rgba.cloned().unwrap_or(Rgba::WHITE),
                    transparent.is_some(),
                );
            }
        }
    }
}

pub(super) fn encode_sprite(
    buffer: &mut Vec<Flat2DData>,
    texture: Handle<Texture>,
    frame: &SpriteFrame,
    transform: &GlobalTransform,
    flip: Option<&Flipped>,
    tint: Rgba,
    transparent: bool,
) {
    let (flip_horizontal, flip_vertical) = match flip {
        Some(Flipped::Horizontal) => (true, false),
        Some(Flipped::Vertical) => (false, true),
        Some(Flipped::Both) => (true, true),
        _ => (false, false),
    };

    let tex_coords = &frame.tex_coords;
    let (uv_left, uv_right) = if flip_horizontal {
        (tex_coords.right, tex_coords.left)
    } else {
        (tex_coords.left, tex_coords.right)
    };
    let (uv_bottom, uv_top) = if flip_vertical {
        (tex_coords.top, tex_coords.bottom)
    } else {
        (tex_coords.bottom, tex_coords.top)
    };

    let dir_x = transform.0.column(0) * frame.width;
    let dir_y = transform.0.column(1) * frame.height;

    // The offsets are negated to shift the sprite left and down relative to the entity, in
    // regards to pivot points. This is the convention adopted in:
    //
    // * libgdx: <https://gamedev.stackexchange.com/q/22553>
    // * godot: <https://godotengine.org/qa/9784>
    let pos = transform.0 * Vector4::new(-frame.offsets[0], -frame.offsets[1], 0.0, 1.0);

    buffer.push(Flat2DData {
        texture,
        dir_x,
        dir_y,
        pos,
        uv_left,
        uv_right,
        uv_top,
        uv_bottom,
        tint,
        transparent,
    });
}
