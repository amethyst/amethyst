use amethyst_assets::Handle;
use amethyst_core::ecs::{Component, DenseVecStorage};
use amethyst_rendy::{SpriteRender, Texture};

/// Image used UI widgets, often as background.
#[derive(Debug, Clone, PartialEq)]
pub enum UiImage {
    /// An image backed by texture handle
    Texture(Handle<Texture>),
    /// An image backed by a texture cropped to specified rectangle
    PartialTexture {
        /// Texture handle
        tex: Handle<Texture>,
        /// Left Texture Coordinate
        left: f32,
        /// Right Texture Coordinate
        right: f32,
        /// Bottom Texture Coordinate
        bottom: f32,
        /// Top Texture Coordinate
        top: f32,
    },
    /// An image backed by a Sprite
    Sprite(SpriteRender),
    /// An Image backed by a 9-sliced texture
    NineSlice {
        ///x starting position on the texture
        x_start: u32,
        /// Y starting position on the texture
        y_start: u32,
        /// Width of the image in the texture
        width: u32,
        /// Height of the image in the texture
        height: u32,
        /// Distance from the left edge of the image for a slice
        left_dist: u32,
        /// Distance from the right edge of the image for a slice
        right_dist: u32,
        /// Distance from the top edge of the image for a slice
        top_dist: u32,
        /// Distance from the bottom edge of the image for a slice
        bottom_dist: u32,
        /// Texture handle
        tex: Handle<Texture>,
        /// Dimensions of the entire texture
        texture_dimensions: [u32; 2],
    },
    /// An image entirely covered by single solid color
    /// This tuple takes linear RGBA. You can convert rgba to linear rgba like so:
    ///
    /// ```
    /// use amethyst_rendy::palette::Srgba;
    /// use amethyst_ui::UiImage;
    ///
    /// let your_red: f32 = 255.;
    /// let your_green: f32 = 160.;
    /// let your_blue: f32 = 122.;
    /// let your_alpha: f32 = 1.0;
    ///
    /// let (r, g, b, a) = Srgba::new(your_red / 255., your_green / 255., your_blue / 255., your_alpha)
    ///     .into_linear()
    ///     .into_components();
    ///
    /// UiImage::SolidColor([r, g, b, a]);
    /// ```
    SolidColor([f32; 4]),
}

impl Component for UiImage {
    type Storage = DenseVecStorage<Self>;
}
