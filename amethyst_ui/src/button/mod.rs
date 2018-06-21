mod builder;
mod system;

pub use self::builder::{UiButtonBuilder, UiButtonBuilderResources};
pub use self::system::UiButtonSystem;

///! A clickable button.
use amethyst_audio::SourceHandle;
use amethyst_core::specs::prelude::{Component, DenseVecStorage};

use amethyst_renderer::TextureHandle;

/// A clickable button, this must be paired with a `UiImage`
/// and this entity must have a child entity with a `UiText`.
pub struct UiButton {
    /// Default text color
    normal_text_color: [f32; 4],
    /// Default image
    normal_image: TextureHandle,
    /// Image used when the mouse hovers over this element
    hover_image: Option<TextureHandle>,
    /// Text color used when this button is hovered over
    hover_text_color: Option<[f32; 4]>,
    /// Image used when button is pressed
    press_image: Option<TextureHandle>,
    /// Text color used when this button is pressed
    press_text_color: Option<[f32; 4]>,
    /// Sound made when this button is hovered over
    hover_sound: Option<SourceHandle>,
    /// Sound made when this button is pressed.
    press_sound: Option<SourceHandle>,
}

impl UiButton {
    /// A constructor for this component.  It's recommended to use
    /// either a prefab or `UiButtonBuilder` rather than this function
    /// if possible.
    pub fn new(
        normal_text_color: [f32; 4],
        normal_image: TextureHandle,
        hover_image: Option<TextureHandle>,
        hover_text_color: Option<[f32; 4]>,
        press_image: Option<TextureHandle>,
        press_text_color: Option<[f32; 4]>,
        hover_sound: Option<SourceHandle>,
        press_sound: Option<SourceHandle>,
    ) -> Self {
        Self {
            normal_text_color,
            normal_image,
            hover_image,
            hover_text_color,
            press_image,
            press_text_color,
            hover_sound,
            press_sound,
        }
    }
}

impl Component for UiButton {
    type Storage = DenseVecStorage<Self>;
}
