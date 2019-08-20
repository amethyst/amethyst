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
        ///x starting position on the texture
        x_start: i32,
        ///y starting position on the texture
        y_start: i32,
        ///width of the image in the texture
        width: i32,
        ///height of the image in the texture
        height: i32,
        ///distance from the left edge of the image for a slice
        left_dist: i32,
        ///distance from the right edge of the image for a slice
        right_dist: i32,
        ///distance from the top edge of the image for a slice
        top_dist: i32,
        ///distance from the bottom edge of the image for a slice
        bottom_dist: i32,
        ///texture handle
        tex: Handle<Texture>,
        ///dimensions of the entire texture
        texture_dimensions: [i32; 2],
    },
    /// An image entirely covered by single solid color
    SolidColor([f32; 4]),
}

impl Component for UiImage {
    type Storage = DenseVecStorage<Self>;
}
