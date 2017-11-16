use shrev::{EventChannel, ReaderId};
use specs::{Component, DenseVecStorage, Fetch, FetchMut, Join, System, WriteStorage};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use unicode_segmentation::UnicodeSegmentation;
use winit::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

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
    /// The current editing cursor position, specified in terms of glyphs, not characters.
    pub cursor_position: isize,
    /// The amount and direction of glyphs highlighted relative to the cursor.
    pub highlight_vector: isize,
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

/// This system processes the underlying UI data as needed.
pub struct UiSystem {
    reader: ReaderId,
}

impl UiSystem {
    /// Initializes a new UiSystem that uses the given reader id.
    pub fn new(reader: ReaderId) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for UiSystem {
    type SystemData = (
        WriteStorage<'a, UiText>,
        WriteStorage<'a, TextEditing>,
        FetchMut<'a, UiFocused>,
        Fetch<'a, EventChannel<Event>>,
    );

    fn run(&mut self, (mut text, mut editable, mut focused, events): Self::SystemData) {
        for text in (&mut text).join() {
            if (*text.text).chars().any(|c| is_combining_mark(c)) {
                let normalized = text.text.nfd().collect::<String>();
                text.text = normalized;
            }
        }
        for event in events.lossy_read(&mut self.reader).unwrap() {
            if let Some((ref mut focused_text, ref mut focused_edit)) =
                focused.entity.and_then(|entity| {
                    text.get_mut(entity)
                        .into_iter()
                        .zip(editable.get_mut(entity).into_iter())
                        .next()
                }) {
                match *event {
                    Event::WindowEvent {
                        event: WindowEvent::ReceivedCharacter(input),
                        ..
                    } => {
                        // Ignore obsolete control characters
                        if input < '\u{8}' || (input > '\u{D}' && input < '\u{20}') {
                            continue;
                        }
                        let deleted = focused_edit.highlight_vector != 0;
                        if deleted {
                            let start = focused_edit
                                .cursor_position
                                .min(focused_edit.cursor_position + focused_edit.highlight_vector)
                                as usize;
                            let end = focused_edit
                                .cursor_position
                                .max(focused_edit.cursor_position + focused_edit.highlight_vector)
                                as usize;
                            let start_delete_byte =
                                focused_text.text.grapheme_indices(true).nth(start).map(|i| i.0);
                            let end_delete_byte = focused_text.text.grapheme_indices(true).nth(end).map(|i| i.0)
                                .unwrap_or(focused_text.text.len());;
                            focused_edit.cursor_position = start as isize;
                            focused_edit.highlight_vector = 0;
                            if let Some(start_delete_byte) = start_delete_byte {
                                focused_text.text.drain(start_delete_byte..end_delete_byte);
                            }
                        }

                        let (start_byte, start_glyph_len) = focused_text
                            .text
                            .grapheme_indices(true)
                            .nth(focused_edit.cursor_position as usize)
                            .map(|i| (i.0, i.1.len()))
                            .unwrap_or_else(|| {
                                // Text is 0 length so has no graphemes
                                let len = focused_text.text.len();
                                if len == 0 {
                                    (0, 0)
                                } else {
                                    // Text has length, so cursor position must be at the end.
                                    (len, 0)
                                }
                            });
                        match input {
                            '\u{8}' /*Backspace*/ => if !deleted {
                                if focused_edit.cursor_position > 0 {
                                    if let Some((byte, len)) = focused_text
                                        .text
                                        .grapheme_indices(true)
                                        .nth(focused_edit.cursor_position as usize - 1)
                                        .map(|i| (i.0, i.1.len())) {
                                            {
                                                focused_text.text.drain(byte..(byte + len));
                                            }
                                            focused_edit.cursor_position -= 1;
                                    }
                                }
                            },
                            '\u{7F}' /*Delete*/ => if !deleted {
                                focused_text.text.drain(start_byte..(start_byte + start_glyph_len));
                            },
                            _ => {
                                focused_text.text.insert(start_byte, input);
                                focused_edit.cursor_position += 1;
                            }
                        }
                    }
                    Event::WindowEvent {
                        event:
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(v_keycode),
                                        modifiers,
                                        ..
                                    },
                                ..
                            },
                        ..
                    } => match v_keycode {
                        VirtualKeyCode::Left => {
                            if focused_edit.cursor_position > 0 {
                                focused_edit.cursor_position -= 1;
                                if modifiers.shift {
                                    focused_edit.highlight_vector += 1;
                                } else {
                                    focused_edit.highlight_vector = 0;
                                }
                            }
                        },
                        VirtualKeyCode::Right => {
                            let glyph_len = focused_text.text.graphemes(true).count();
                            if (focused_edit.cursor_position as usize) < glyph_len {
                                focused_edit.cursor_position += 1;
                                if modifiers.shift {
                                    focused_edit.highlight_vector -= 1;
                                } else {
                                    focused_edit.highlight_vector = 0;
                                }
                            }
                        },
                        VirtualKeyCode::A => {
                            if modifiers.ctrl {
                                let glyph_len = focused_text.text.graphemes(true).count();
                                focused_edit.cursor_position = glyph_len as isize;
                                focused_edit.highlight_vector = -(glyph_len as isize);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}
