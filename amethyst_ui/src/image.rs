use amethyst_renderer::TextureHandle;
use specs::{Component, DenseVecStorage};

/// A component with the texture to display in this entities UiTransform
pub struct UiImage {
    /// The texture to display
    pub texture: TextureHandle,
}

impl Component for UiImage {
    type Storage = DenseVecStorage<Self>;
}
