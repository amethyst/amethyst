use amethyst_assets::Handle;
use amethyst_core::ecs::{Component, DenseVecStorage};
use amethyst_rendy::sprite::TextureCoordinates;
use amethyst_rendy::{SpriteRender, Texture};

/// Image used UI widgets, often as background.
#[derive(Debug, Clone, PartialEq)]
pub enum UiImage {
    /// An image backed by texture handle
    Texture(Handle<Texture>),
    /// An image backed by a texture cropped to specified rectangle
    PartialTexture(Handle<Texture>, TextureCoordinates),
    /// An image backed by a Sprite
    Sprite(SpriteRender),
    /// An Image backed by a 9-sliced texture
    NineSlice {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        left_dist: i32,
        right_dist: i32,
        top_dist: i32,
        bottom_dist: i32,
        texture: Handle<Texture>,
        texture_dimensions: [i32; 2],
    },
    /// An image entirely covered by single solid color
    SolidColor([f32; 4]),
}

impl Component for UiImage {
    type Storage = DenseVecStorage<Self>;
}
