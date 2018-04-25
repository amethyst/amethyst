use amethyst_core::specs::prelude::{Component, VecStorage};
use amethyst_renderer::TextureHandle;

/// A component with the texture to display in this entity's `UiTransform`
#[derive(Clone)]
pub struct UiImage {
    /// The texture to display
    pub texture: TextureHandle,
}

impl Component for UiImage {
    type Storage = VecStorage<Self>;
}
