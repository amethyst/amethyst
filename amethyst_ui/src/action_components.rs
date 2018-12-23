use amethyst_audio::SourceHandle;
use amethyst_core::specs::prelude::{Component, DenseVecStorage};
use amethyst_renderer::TextureHandle;

/// When this component is added to a UI element with a `TextureHandle`
/// it will change that image based on mouse interaction.
/// Requires `MouseReactive`.
pub struct OnUiActionImage {
    /// Default image
    pub(crate) normal_image: Option<TextureHandle>,
    /// Image used when the mouse hovers over this element
    pub(crate) hover_image: Option<TextureHandle>,
    /// Image used when element is pressed
    pub(crate) press_image: Option<TextureHandle>,
}

impl OnUiActionImage {
    /// A constructor for this component
    pub fn new(
        normal_image: Option<TextureHandle>,
        hover_image: Option<TextureHandle>,
        press_image: Option<TextureHandle>,
    ) -> Self {
        Self {
            normal_image,
            hover_image,
            press_image,
        }
    }
}

impl Component for OnUiActionImage {
    type Storage = DenseVecStorage<Self>;
}

/// When this component is added to a UI element
/// it will play sounds based on mouse interaction.
/// Requires `MouseReactive`.
pub struct OnUiActionSound {
    /// Sound made when this button is hovered over
    pub(crate) hover_sound: Option<SourceHandle>,
    /// Sound made when this button is pressed.
    pub(crate) press_sound: Option<SourceHandle>,
    /// Sound made when this button is released.
    pub(crate) release_sound: Option<SourceHandle>,
}

impl OnUiActionSound {
    /// A constructor for this component
    pub fn new(
        hover_sound: Option<SourceHandle>,
        press_sound: Option<SourceHandle>,
        release_sound: Option<SourceHandle>,
    ) -> Self {
        Self {
            hover_sound,
            press_sound,
            release_sound,
        }
    }
}

impl Component for OnUiActionSound {
    type Storage = DenseVecStorage<Self>;
}
