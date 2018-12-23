mod actions;
mod builder;
mod retrigger;
mod system;

pub use self::actions::{UiButtonAction, UiButtonActionType};
pub use self::builder::{UiButtonBuilder, UiButtonBuilderResources};
pub use self::retrigger::{UiButtonActionRetrigger, UiButtonActionRetriggerSystem};
pub use self::system::UiButtonSystem;
///! A clickable button.
use amethyst_core::specs::prelude::{Component, DenseVecStorage};

/// A clickable button, this must be paired with a `TextureHandle`
/// and this entity must have a child entity with a `UiText`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiButton {
    /// Default text color
    text_color: [f32; 4],
}

impl UiButton {
    /// A constructor for this component.  It's recommended to use
    /// either a prefab or `UiButtonBuilder` rather than this function
    /// if possible.
    pub fn new(text_color: [f32; 4]) -> Self {
        Self { text_color }
    }
}

impl Component for UiButton {
    type Storage = DenseVecStorage<Self>;
}
