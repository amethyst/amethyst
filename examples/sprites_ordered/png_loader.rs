use amethyst::{
    assets::{AssetStorage, Loader},
    prelude::*,
    renderer::{PngFormat, Texture, TextureHandle},
};

/// Returns a `TextureHandle` to the image.
///
/// # Parameters
///
/// * `name`: Path to the sprite sheet.
/// * `world`: `World` that stores resources.
pub fn load<N>(name: N, world: &World) -> TextureHandle
where
    N: Into<String>,
{
    let loader = world.read_resource::<Loader>();
    loader.load(
        name,
        PngFormat,
        Default::default(),
        (),
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}
