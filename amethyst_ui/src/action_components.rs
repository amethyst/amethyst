use amethyst_audio::SourceHandle;
use amethyst_rendy::TextureHandle;

/// When this component is added to a UI element with a `TextureHandle`
/// it will change that image based on mouse interaction.
/// Requires `MouseReactive`.
pub struct OnUiActionImage {
    /// Default image
    pub(crate) normal_image: Option<UiImage>,
    /// Image used when the mouse hovers over this element
    pub(crate) hover_image: Option<UiImage>,
    /// Image used when element is pressed
    pub(crate) press_image: Option<UiImage>,
}

impl OnUiActionImage {
    /// A constructor for this component
    pub fn new(
        normal_image: Option<UiImage>,
        hover_image: Option<UiImage>,
        press_image: Option<UiImage>,
    ) -> Self {
        Self {
            normal_image,
            hover_image,
            press_image,
        }
    }
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