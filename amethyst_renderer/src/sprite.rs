use amethyst_assets::{Asset, Handle};
use amethyst_core::specs::VecStorage;

/// An asset handle to sprite sheet metadata.
pub type SpriteSheetHandle = Handle<SpriteSheet>;

/// Meta data for a sprite sheet texture.
///
/// Contains a handle to the texture and the sprite coordinates on the texture.
#[derive(Clone, Debug)]
pub struct SpriteSheet {
    /// Index of the texture for this sprite sheet.
    pub index: usize,
    /// A list of sprites in this sprite sheet.
    pub sprites: Vec<Sprite>,
}

impl Asset for SpriteSheet {
    const NAME: &'static str = "renderer::Sprite";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

/// A description of a frame in a sprite sheet.
///
/// These should be in normalized coordinates:
///
/// * X axis: 0.0 is the left side and 1.0 is the right side.
/// * Y axis: 0.0 is the top and 1.0 is the botoom.
#[derive(Clone, Debug)]
pub struct Sprite {
    /// Normalized left x coordinate
    pub left: f32,
    /// Normalized top y coordinate
    pub top: f32,
    /// Normalized right x coordinate
    pub right: f32,
    /// Normalized bottom y coordinate
    pub bottom: f32,
}
