//! Module holding the components related to text and text editing.

use amethyst_core::{
    ecs::*,
    shrev::{EventChannel, ReaderId},
    timing::Time,
};
use amethyst_window::ScreenDimensions;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use winit::{ElementState, Event, MouseButton, WindowEvent};

use super::*;
use crate::Anchor;
use amethyst_core::ecs::systems::ParallelRunnable;

/// How lines should behave when they are longer than the maximum line length.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum LineMode {
    /// Single line. It ignores line breaks.
    Single,
    /// Multiple lines. The text will automatically wrap when exceeding the maximum width.
    Wrap,
}

/// A component used to display text in this entity's UiTransform
#[derive(Clone, Derivative, Serialize)]
#[derivative(Debug)]
pub struct UiText {
    /// The string rendered by this.
    pub text: String,
    /// The height of a line of text in pixels.
    pub font_size: f32,
    /// The color of the rendered text, using a range of 0.0 to 1.0 per channel.
    pub color: [f32; 4],
    /// The font used for rendering.
    #[serde(skip)]
    pub font: FontHandle,
    /// If true this will be rendered as dots instead of the text.
    pub password: bool,
    /// How the text should handle new lines.
    pub line_mode: LineMode,
    /// How to align the text within its `UiTransform`.
    pub align: Anchor,
    /// Cached glyph positions including invisible characters, used to process mouse highlighting.
    #[serde(skip)]
    pub(crate) cached_glyphs: Vec<CachedGlyph>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CachedGlyph {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) advance_width: f32,
}

impl UiText {
    /// Initializes a new UiText
    ///
    /// # Parameters
    ///
    /// * `font`: A handle to a `Font` asset
    /// * `text`: The glyphs to render
    /// * `color`: RGBA color with a maximum of 1.0 and a minimum of 0.0 for each channel
    /// * `font_size`: A uniform scale applied to the glyphs
    /// * `line_mode`: Text mode allowing single line or multiple lines
    /// * `align`: Text alignment within its `UiTransform`
    pub fn new(
        font: FontHandle,
        text: String,
        color: [f32; 4],
        font_size: f32,
        line_mode: LineMode,
        align: Anchor,
    ) -> UiText {
        UiText {
            text,
            color,
            font_size,
            font,
            password: false,
            line_mode,
            align,
            cached_glyphs: Vec::new(),
        }
    }
}

/// If this component is attached to an entity with a UiText then that UiText is editable.
/// This component also controls how that editing works.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextEditing {
    /// The current editing cursor position, specified in terms of glyphs, not characters.
    pub cursor_position: isize,
    /// The maximum graphemes permitted in this string.
    pub max_length: usize,
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
    pub fn new(
        max_length: usize,
        selected_text_color: [f32; 4],
        selected_background_color: [f32; 4],
        use_block_cursor: bool,
    ) -> TextEditing {
        TextEditing {
            cursor_position: 0,
            max_length,
            highlight_vector: 0,
            selected_text_color,
            selected_background_color,
            use_block_cursor,
            cursor_blink_timer: 0.0,
        }
    }
}

/// This system processes the underlying UI data as needed.
#[derive(Debug)]
pub struct TextEditingMouseSystem {
    /// A reader for winit events.
    event_reader: ReaderId<Event>,
    /// This is set to true while the left mouse button is pressed.
    left_mouse_button_pressed: bool,
    /// The screen coordinates of the mouse
    mouse_position: (f32, f32),
}

impl TextEditingMouseSystem {
    /// Creates a new instance of this system
    pub fn new(event_reader: ReaderId<Event>) -> Self {
        Self {
            event_reader,
            left_mouse_button_pressed: false,
            mouse_position: (0., 0.),
        }
    }

    pub fn build(&mut self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TextEditingMouseSystem")
                .read_resource::<Time>()
                .read_resource::<EventChannel<Event>>()
                .read_resource::<ScreenDimensions>()
                .with_query(<Write<UiText>>::query())
                .with_query(<(Write<TextEditing>, &Selected)>::query())
                .with_query(<(Write<UiText>, Write<TextEditing>, Option<&Selected>)>::query())
                .build(move |_commands, world,
                             (time, events, screen_dimensions),
                             (texts, selected_text_editings, maybe_selected_texts ) | {
                    // Normalize text to ensure we can properly count the characters.
                    // TODO: Possible improvement to be made if this can be moved only when inserting characters into ui text.
                    texts.for_each_mut(world, |mut text| {
                        if (*text.text).chars().any(is_combining_mark) {
                            let normalized = text.text.nfd().collect::<String>();
                            text.text = normalized;
                        }
                    });

                    // TODO: Finish TextEditingCursorSystem and remove this
                    selected_text_editings.for_each_mut(world, |text_editing, _| {
                        text_editing.cursor_blink_timer += time.delta_real_seconds();
                        if text_editing.cursor_blink_timer >= 0.5 {
                            text_editing.cursor_blink_timer = 0.0;
                        }
                    });

                    let mut just_pressed = false;
                    let mut moved_while_pressed = false;

                    let event_reader = &mut self.event_reader;

                    // Process only if an editable text is selected.
                    for event in events.read(event_reader) {
                        // Process events for the whole UI.
                        match *event {
                            Event::WindowEvent {
                                event: WindowEvent::CursorMoved { position, .. },
                                ..
                            } => {
                                let hidpi = screen_dimensions.hidpi_factor() as f32;
                                self.mouse_position = (
                                    position.x as f32 * hidpi,
                                    (screen_dimensions.height() - position.y as f32) * hidpi,
                                );
                                if self.left_mouse_button_pressed {
                                    moved_while_pressed = true;
                                }
                            }
                            Event::WindowEvent {
                                event:
                                WindowEvent::MouseInput {
                                    button: MouseButton::Left,
                                    state,
                                    ..
                                },
                                ..
                            } => match state {
                                ElementState::Pressed => {
                                    just_pressed = true;
                                    self.left_mouse_button_pressed = true;
                                }
                                ElementState::Released => {
                                    self.left_mouse_button_pressed = false;
                                }
                            },
                            _ => {}
                        }
                    }

                    maybe_selected_texts.for_each_mut(world, |(mut text, mut text_editing, selected)| {
                        if selected.is_none() {
                            // If an editable text field is no longer selected, we should reset
                            // the highlight vector.
                            text_editing.highlight_vector = 0;
                        } else if just_pressed {
                            // If we focused an editable text field be sure to position the cursor
                            // in it.
                            let (mouse_x, mouse_y) = self.mouse_position;
                            text_editing.highlight_vector = 0;
                            text_editing.cursor_position =
                                closest_glyph_index_to_mouse(mouse_x, mouse_y, &text.cached_glyphs);
                            text_editing.cursor_blink_timer = 0.0;

                            // The end of the text, while not a glyph, is still something
                            // you'll likely want to click your cursor to, so if the cursor is
                            // near the end of the text, check if we should put it at the end
                            // of the text.
                            if should_advance_to_end(mouse_x, text_editing, text) {
                                text_editing.cursor_position += 1;
                            }
                        } else if moved_while_pressed {
                            let (mouse_x, mouse_y) = self.mouse_position;
                            text_editing.highlight_vector =
                                closest_glyph_index_to_mouse(mouse_x, mouse_y, &text.cached_glyphs)
                                    - text_editing.cursor_position;
                            // The end of the text, while not a glyph, is still something
                            // you'll likely want to click your cursor to, so if the cursor is
                            // near the end of the text, check if we should put it at the end
                            // of the text.
                            if should_advance_to_end(mouse_x, text_editing, text) {
                                text_editing.highlight_vector += 1;
                            }
                        }
                    });
                }
                )
        )
    }
}

fn should_advance_to_end(mouse_x: f32, text_editing: &mut TextEditing, text: &mut UiText) -> bool {
    let cursor_pos = text_editing.cursor_position + text_editing.highlight_vector;
    let len = text.cached_glyphs.len() as isize;
    if cursor_pos + 1 == len {
        if let Some(last_glyph) = text.cached_glyphs.last() {
            if mouse_x - last_glyph.x > last_glyph.advance_width / 2.0 {
                return true;
            }
        }
    }

    false
}

fn closest_glyph_index_to_mouse(mouse_x: f32, mouse_y: f32, glyphs: &[CachedGlyph]) -> isize {
    glyphs
        .iter()
        .enumerate()
        .min_by(|(_, g1), (_, g2)| {
            let dist = |g: &CachedGlyph| {
                let dx = g.x - mouse_x;
                let dy = g.y - mouse_y;
                dx * dx + dy * dy
            };
            dist(g1).partial_cmp(&dist(g2)).expect("Unexpected NaN!")
        })
        .map(|(i, _)| i)
        .unwrap_or(0) as isize
}
