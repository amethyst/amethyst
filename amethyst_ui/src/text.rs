use std::ops::Range;

use amethyst_core::timing::Time;
use clipboard::{ClipboardContext, ClipboardProvider};
use shrev::{EventChannel, ReaderId};
use specs::{Component, DenseVecStorage, Fetch, FetchMut, Join, System, WriteStorage};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;
use unicode_segmentation::UnicodeSegmentation;
use winit::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};

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
    /// If true this will be rendered as dots instead of the text.
    pub password: bool,
    /// Cached FontHandle, used to detect changes to the font.
    pub(crate) cached_font: FontHandle,
    /// Cached id used to retrieve the `GlyphBrush` in the `UiPass`.
    pub(crate) brush_id: Option<u32>,
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
            password: false,
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

    /// This value is used to control cursor blinking.
    ///
    /// When it is greater than 0.5 / CURSOR_BLINK_RATE the cursor should not display, when it
    /// is greater than or equal to 1.0 / CURSOR_BLINK_RATE it should be reset to 0.  When the
    /// player types it should be reset to 0.
    pub(crate) cursor_blink_timer: f32,
}

impl TextEditing {
    /// Create a new TextEditing Component
    pub fn new(selected_text_color: [f32; 4], selected_background_color: [f32; 4], use_block_cursor: bool) -> TextEditing {
        TextEditing {
            cursor_position: 0,
            highlight_vector: 0,
            selected_text_color,
            selected_background_color,
            use_block_cursor,
            cursor_blink_timer: 0.0,
        }
    }
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
        Fetch<'a, Time>,
    );

    fn run(&mut self, (mut text, mut editable, focused, events, time): Self::SystemData) {
        for text in (&mut text).join() {
            if (*text.text).chars().any(|c| is_combining_mark(c)) {
                let normalized = text.text.nfd().collect::<String>();
                text.text = normalized;
            }
        }
        let mut focused_text_edit = focused.entity.and_then(|entity| {
            text.get_mut(entity)
                .into_iter()
                .zip(editable.get_mut(entity).into_iter())
                .next()
        });
        if let Some((ref mut _focused_text, ref mut focused_edit)) = focused_text_edit {
            focused_edit.cursor_blink_timer += time.delta_real_seconds();
            if focused_edit.cursor_blink_timer >= 1.0 / CURSOR_BLINK_RATE {
                focused_edit.cursor_blink_timer = 0.0;
            }
        }

        for event in events.lossy_read(&mut self.reader).unwrap() {
            if let Some((ref mut focused_text, ref mut focused_edit)) = focused_text_edit {
                match *event {
                    Event::WindowEvent {
                        event: WindowEvent::ReceivedCharacter(input),
                        ..
                    } => {
                        // Ignore obsolete control characters
                        if input < '\u{8}' || (input > '\u{D}' && input < '\u{20}') {
                            continue;
                        }
                        // Since delete character isn't emitted on windows, ignore it too.
                        // We'll handle this with the KeyboardInput event instead.
                        if input == '\u{7F}' {
                            continue;
                        }
                        focused_edit.cursor_blink_timer = 0.0;
                        let deleted = delete_highlighted(focused_edit, focused_text);
                        let start_byte = focused_text
                            .text
                            .grapheme_indices(true)
                            .nth(focused_edit.cursor_position as usize)
                            .map(|i| i.0)
                            .unwrap_or_else(|| {
                                // We are either in a 0 length string, or at the end of a string
                                // This line returns the correct byte index for both.
                                focused_text.text.len()
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
                        VirtualKeyCode::Home => {
                            focused_edit.highlight_vector = if modifiers.shift {
                                focused_edit.cursor_position
                            } else {
                                0
                            };
                            focused_edit.cursor_position = 0;
                            focused_edit.cursor_blink_timer = 0.0;
                        }
                        VirtualKeyCode::End => {
                            let glyph_len = focused_text.text.graphemes(true).count() as isize;
                            focused_edit.highlight_vector = if modifiers.shift {
                                focused_edit.cursor_position - glyph_len
                            } else {
                                0
                            };
                            focused_edit.cursor_position = glyph_len;
                            focused_edit.cursor_blink_timer = 0.0;
                        }
                        VirtualKeyCode::Delete => {
                            if !delete_highlighted(focused_edit, focused_text) {
                                if let Some((start_byte, start_glyph_len)) = focused_text
                                    .text
                                    .grapheme_indices(true)
                                    .nth(focused_edit.cursor_position as usize)
                                    .map(|i| (i.0, i.1.len()))
                                {
                                    focused_edit.cursor_blink_timer = 0.0;
                                    focused_text
                                        .text
                                        .drain(start_byte..(start_byte + start_glyph_len));
                                }
                            }
                        }
                        VirtualKeyCode::Left => if focused_edit.highlight_vector == 0
                            || modifiers.shift
                        {
                            if focused_edit.cursor_position > 0 {
                                let delta = if ctrl_or_cmd(&modifiers) {
                                    let mut graphemes = 0;
                                    for word in focused_text.text.split_word_bounds() {
                                        let word_graphemes = word.graphemes(true).count() as isize;
                                        if graphemes + word_graphemes
                                            >= focused_edit.cursor_position
                                        {
                                            break;
                                        }
                                        graphemes += word_graphemes;
                                    }
                                    focused_edit.cursor_position - graphemes
                                } else {
                                    1
                                };
                                focused_edit.cursor_position -= delta;
                                if modifiers.shift {
                                    focused_edit.highlight_vector += delta;
                                }
                                focused_edit.cursor_blink_timer = 0.0;
                            }
                        } else {
                            focused_edit.cursor_position = focused_edit
                                .cursor_position
                                .min(focused_edit.cursor_position + focused_edit.highlight_vector);
                            focused_edit.highlight_vector = 0;
                        },
                        VirtualKeyCode::Right => {
                            if focused_edit.highlight_vector == 0 || modifiers.shift {
                                let glyph_len = focused_text.text.graphemes(true).count();
                                if (focused_edit.cursor_position as usize) < glyph_len {
                                    let delta = if ctrl_or_cmd(&modifiers) {
                                        let mut graphemes = 0;
                                        for word in focused_text.text.split_word_bounds() {
                                            graphemes += word.graphemes(true).count() as isize;
                                            if graphemes > focused_edit.cursor_position {
                                                break;
                                            }
                                        }
                                        graphemes - focused_edit.cursor_position
                                    } else {
                                        1
                                    };
                                    focused_edit.cursor_position += delta;
                                    if modifiers.shift {
                                        focused_edit.highlight_vector -= delta;
                                    }
                                    focused_edit.cursor_blink_timer = 0.0;
                                }
                            } else {
                                focused_edit.cursor_position = focused_edit.cursor_position.max(
                                    focused_edit.cursor_position + focused_edit.highlight_vector,
                                );
                                focused_edit.highlight_vector = 0;
                            }
                        },
                        VirtualKeyCode::A => if ctrl_or_cmd(&modifiers) {
                            let glyph_len = focused_text.text.graphemes(true).count() as isize;
                            focused_edit.cursor_position = glyph_len;
                            focused_edit.highlight_vector = -glyph_len;
                        },
                        VirtualKeyCode::X => if ctrl_or_cmd(&modifiers) {
                            let new_clip = extract_highlighted(focused_edit, focused_text);
                            if new_clip.len() > 0 {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                ctx.set_contents(new_clip).unwrap();
                            }
                        },
                        VirtualKeyCode::C => if ctrl_or_cmd(&modifiers) {
                            let new_clip = read_highlighted(focused_edit, focused_text);
                            if new_clip.len() > 0 {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                ctx.set_contents(new_clip.to_owned()).unwrap();
                            }
                        },
                        VirtualKeyCode::V => if ctrl_or_cmd(&modifiers) {
                            delete_highlighted(focused_edit, focused_text);
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            if let Ok(contents) = ctx.get_contents() {
                                let index = cursor_byte_index(focused_edit, focused_text);
                                focused_text.text.insert_str(index, &contents);
                                focused_edit.cursor_position += contents.graphemes(true).count() as isize;
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

/// Returns if the command key is down on OSX, and the CTRL key for everything else.
fn ctrl_or_cmd(modifiers: &ModifiersState) -> bool {
    (cfg!(target_os = "macos") && modifiers.logo)
        || (cfg!(not(target_os = "macos")) && modifiers.ctrl)
}

fn read_highlighted<'a>(edit: &TextEditing, text: &'a UiText) -> &'a str {
    let range = highlighted_bytes(edit, text);
    &text.text[range]
}

/// Removes the highlighted text and returns it in a String.
fn extract_highlighted(edit: &mut TextEditing, text: &mut UiText) -> String {
    let range = highlighted_bytes(edit, text);
    edit.cursor_position = range.start as isize;
    edit.highlight_vector = 0;
    text.text.drain(range).collect::<String>()
}

/// Removes the highlighted text and returns true if anything was deleted..
fn delete_highlighted(edit: &mut TextEditing, text: &mut UiText) -> bool {
    if edit.highlight_vector != 0 {
        let range = highlighted_bytes(edit, text);
        edit.cursor_position = range.start as isize;
        edit.highlight_vector = 0;
        text.text.drain(range);
        return true;
    }
    false
}

// Gets the byte index of the cursor.
fn cursor_byte_index(edit: &TextEditing, text: &UiText) -> usize {
    text.text.grapheme_indices(true).nth(edit.cursor_position as usize).map(|i| i.0).unwrap_or(text.text.len())
}

/// Returns the byte indices that are highlighted in the string.
fn highlighted_bytes(edit: &TextEditing, text: &UiText) -> Range<usize> {
    let start = edit.cursor_position
        .min(edit.cursor_position + edit.highlight_vector) as usize;
    let end = edit.cursor_position
        .max(edit.cursor_position + edit.highlight_vector) as usize;
    let start_byte = text.text.grapheme_indices(true).nth(start).map(|i| i.0).unwrap_or(text.text.len());
    let end_byte = text.text
        .grapheme_indices(true)
        .nth(end)
        .map(|i| i.0)
        .unwrap_or(text.text.len());
    start_byte..end_byte
}
