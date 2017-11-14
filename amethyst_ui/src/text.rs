use std::ops::Range;

use specs::{Component, DenseVecStorage, Join, System, WriteStorage};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use super::*;

/// A component used to display text in this entity's UiTransform
pub struct UiText {
    /// The string rendered by this.
    pub text: String,
    /// The height of a line of text in pixels.
    pub font_size: f32,
    /// The color of the rendered text, using a range of 0.0 to 1.0 per channel.
    pub color: [f32; 4],
    /// The font used for rendering.
    pub font: FontHandle,
    /// Cached FontHandle, used to detect changes to the font.
    pub(crate) cached_font: FontHandle,
    /// Cached id used to retrieve the `GlyphBrush` in the `UiPass`.
    pub(crate) brush_id: Option<usize>,
}

impl UiText {
    /// Initializes a new UiText
    ///
    /// # Parameters
    ///
    /// * `font`: A handle to a `Font` asset
    /// * `text`: the glyphs to render
    /// * `color`: RGBA color with a maximum of 1.0 and a minimum of 0.0 for each channel
    /// * `font_size`: a uniform scale applied to the glyphs
    pub fn new(font: FontHandle, text: String, color: [f32; 4], font_size: f32) -> UiText {
        UiText {
            text,
            color,
            font_size,
            font: font.clone(),
            cached_font: font,
            brush_id: None,
        }
    }
}

impl Component for UiText {
    type Storage = DenseVecStorage<Self>;
}

/// If this component is attached to an entity with a UiText then that UiText is editable.
/// This component also controls how that editing works.
pub struct TextEditing {
    /// If the entity contains a UiText this is the beginning and end of the text currently
    /// selected.  If this range has length 0 then start is a cursor position.
    pub text_selected: Range<usize>,
    /// The color of the text itself when highlighted.
    pub selected_text_color: [f32; 4],
    /// The text background color when highlighted.
    pub selected_background_color: [f32; 4],
    /// If this is true the text will use a block cursor for editing.  Otherwise this uses a
    /// standard line cursor.  This is not recommended if your font is not monospace.
    pub use_block_cursor: bool,
}

impl Component for TextEditing {
    type Storage = DenseVecStorage<Self>;
}

/// This system normalizes text in UiText so that it can be rendered properly.  It's added
/// automatically by the bundle.
pub struct TextNormalizer;

impl<'a> System<'a> for TextNormalizer {
    type SystemData = WriteStorage<'a, UiText>;

    fn run(&mut self, mut text: Self::SystemData) {
        for text in (&mut text).join() {
            if (*text.text).chars().any(|c| is_combining_mark(c)) {
                let normalized = text.text.nfd().collect::<String>();
                text.text = normalized;
            }
        }
    }
}
