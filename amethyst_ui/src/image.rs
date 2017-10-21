use amethyst_assets::Handle;
use amethyst_renderer::Texture;
use specs::{Component, DenseVecStorage};

/// A component with the texture to display in this entities UiTransform
pub struct UiImage {
    /// The texture to display
    pub texture: Handle<Texture>,

    /// When this is false the image will be stretched or compressed to fit the bounding
    /// `UiTransform`, if it's true then the image will retain its dimensions, either being cut off
    /// if the `UiTransform` is too small, or simply not filling the space if the `UiTransform` is
    /// too large.
    pub preserve_aspect_ratio: bool,
}

impl Component for UiImage {
    type Storage = DenseVecStorage<Self>;
}
