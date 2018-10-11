mod builder;
mod system;

pub use self::builder::{UiButtonBuilder, UiButtonBuilderResources};
pub use self::system::UiButtonSystem;
///! A clickable button.
use amethyst_core::specs::prelude::{Component, DenseVecStorage};

/// A clickable button, this must be paired with a `UiImage`
/// and this entity must have a child entity with a `UiText`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiButton {
    /// Default text color
    normal_text_color: [f32; 4],
    /// Text color used when this button is hovered over
    hover_text_color: Option<[f32; 4]>,
    /// Text color used when this button is pressed
    press_text_color: Option<[f32; 4]>,
}

impl UiButton {
    /// A constructor for this component.  It's recommended to use
    /// either a prefab or `UiButtonBuilder` rather than this function
    /// if possible.
    pub fn new(
        normal_text_color: [f32; 4],
        hover_text_color: Option<[f32; 4]>,
        press_text_color: Option<[f32; 4]>,
    ) -> Self {
        Self {
            normal_text_color,
            hover_text_color,
            press_text_color,
        }
    }
}

impl Component for UiButton {
    type Storage = DenseVecStorage<Self>;
}
