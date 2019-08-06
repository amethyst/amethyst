use amethyst_audio::SourceHandle;
use amethyst_core::ecs::prelude::{Component, DenseVecStorage};
use amethyst_rendy::TextureHandle;

/// When this component is added to a UI element with a `TextureHandle`
/// it will change that image based on mouse interaction.
/// Requires `MouseReactive`.
#[derive(new)]
pub struct OnUiActionImage {
    /// Default image
    pub(crate) normal_image: Option<UiImage>,
    /// Image used when the mouse hovers over this element
    pub(crate) hover_image: Option<UiImage>,
    /// Image used when element is pressed
    pub(crate) press_image: Option<UiImage>,
}

impl Component for OnUiActionImage {
    type Storage = DenseVecStorage<Self>;
}

/// When this component is added to a UI element
/// it will play sounds based on mouse interaction.
/// Requires `MouseReactive`.
#[derive(new)]
pub struct OnUiActionSound {
    /// Sound made when this button is hovered over
    pub(crate) hover_sound: Option<SourceHandle>,
    /// Sound made when this button is pressed.
    pub(crate) press_sound: Option<SourceHandle>,
    /// Sound made when this button is released.
    pub(crate) release_sound: Option<SourceHandle>,
}

impl Component for OnUiActionSound {
    type Storage = DenseVecStorage<Self>;
}
