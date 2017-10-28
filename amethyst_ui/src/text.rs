use amethyst_renderer::TextureHandle;
use rusttype::Font;
use specs::{Component, DenseVecStorage};

/// A component used to display text in this entities UiTransform
pub struct UiText {
    /// The texture that text is rendered onto.
    texture: TextureHandle,
    /// The font used to display the text.
    font: Font<'static>,
    /// The text being displayed
    text: String,
    /// The color of the text being displayed
    color: [f32; 4],
    /// This is true if the texture needs to be re-rendered
    dirty: bool,
}

impl UiText {

}

impl Component for UiText {
    type Storage = DenseVecStorage<Self>;
}
