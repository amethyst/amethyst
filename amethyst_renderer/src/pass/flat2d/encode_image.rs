use super::Flat2DData;
use crate::{Flipped, Hidden, HiddenPropagate, Rgba, Texture, Transparent};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    nalgebra::Vector4,
    specs::{Component, Join, Read, ReadStorage, System, VecStorage, Write},
    GlobalTransform,
};

/// A component that guides `DrawFlat2D` pass encoding for rendering a standalone texture as plain image.
#[derive(Clone, Debug)]
pub struct RenderImageFlat2D(pub Handle<Texture>);

impl Component for RenderImageFlat2D {
    type Storage = VecStorage<Self>;
}

/// An encoder system that prepares entities with `RenderImageFlat2D` component
/// to be drawn using `DrawFlat2D` render pass.
#[derive(Clone, Debug, Default)]
pub struct Flat2DImageEncoder;
impl<'a> System<'a> for Flat2DImageEncoder {
    type SystemData = (
        Write<'a, Vec<Flat2DData>>,
        ReadStorage<'a, RenderImageFlat2D>,
        Read<'a, AssetStorage<Texture>>,
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
            if let Some(tex) = storage.get(&render.0) {
                encode_image(
                    &mut buffer,
                    tex,
                    render.0.clone(),
                    transform,
                    flip,
                    rgba.cloned().unwrap_or(Rgba::WHITE),
                    transparent.is_some(),
                );
            }
        }
    }
}

fn encode_image(
    buffer: &mut Vec<Flat2DData>,
    texture: &Texture,
    texture_handle: Handle<Texture>,
    transform: &GlobalTransform,
    flip: Option<&Flipped>,
    tint: Rgba,
    transparent: bool,
) {
    let (width, height) = texture.size();

    let (uv_left, uv_right, uv_bottom, uv_top) = match flip {
        Some(Flipped::Horizontal) => (1.0, 0.0, 0.0, 1.0),
        Some(Flipped::Vertical) => (0.0, 1.0, 1.0, 0.0),
        Some(Flipped::Both) => (1.0, 0.0, 1.0, 0.0),
        _ => (0.0, 1.0, 0.0, 1.0),
    };

    let dir_x = transform.0.column(0) * (width as f32);
    let dir_y = transform.0.column(1) * (height as f32);
    let pos = transform.0 * Vector4::new(1.0, 1.0, 0.0, 1.0);

    buffer.push(Flat2DData {
        texture: texture_handle,
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
