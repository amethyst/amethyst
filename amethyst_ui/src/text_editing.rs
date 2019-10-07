use std::ops::Range;

use clipboard::{ClipboardContext, ClipboardProvider};
use log::error;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use unicode_segmentation::UnicodeSegmentation;
use winit::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};

use amethyst_core::{
    ecs::prelude::{
        Entities, Join, Read, ReadStorage, System, SystemData, World, Write, WriteStorage,
    },
    shrev::{EventChannel, ReaderId},
    SystemDesc,
};
use amethyst_derive::SystemDesc;

use crate::{LineMode, Selected, TextEditing, UiEvent, UiEventType, UiText};

/// System managing the keyboard inputs for the editable text fields.
/// ## Features
/// * Adds and removes text.
/// * Moves selection cursor.
/// * Grows and shrinks selected text zone.
#[derive(Debug, SystemDesc)]
#[system_desc(name(TextEditingInputSystemDesc))]
pub struct TextEditingInputSystem {
    /// A reader for winit events.
    #[system_desc(event_channel_reader)]
    reader: ReaderId<Event>,
}

impl TextEditingInputSystem {
    /// Creates a new instance of this system
    pub fn new(reader: ReaderId<Event>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for TextEditingInputSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiText>,
        WriteStorage<'a, TextEditing>,
        ReadStorage<'a, Selected>,
        Read<'a, EventChannel<Event>>,
        Write<'a, EventChannel<UiEvent>>,
    );

    fn run(
        &mut self,
        (entities, mut texts, mut editables, selecteds, events, mut edit_events): Self::SystemData,
    ) {
        for text in (&mut texts).join() {
            if (*text.text).chars().any(is_combining_mark) {
                let normalized = text.text.nfd().collect::<String>();
                text.text = normalized;
            }
        }

        for event in events.read(&mut self.reader) {
            // Process events for the focused text element
            if let Some((entity, ref mut focused_text, ref mut focused_edit, _)) =
                (&*entities, &mut texts, &mut editables, &selecteds)
                    .join()
                    .next()
            {
                match *event {
                    Event::WindowEvent {
                        event: WindowEvent::ReceivedCharacter(input),
                        ..
                    } => {
                        if should_skip_char(input) {
                            continue;
                        }
                        focused_edit.cursor_blink_timer = 0.0;
                        delete_highlighted(focused_edit, focused_text);
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
                        if focused_text.text.graphemes(true).count() < focused_edit.max_length {
                            focused_text.text.insert(start_byte, input);
                            focused_edit.cursor_position += 1;

                            edit_events
                                .single_write(UiEvent::new(UiEventType::ValueChange, entity));
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
                        VirtualKeyCode::Home | VirtualKeyCode::Up => {
                            focused_edit.highlight_vector = if modifiers.shift {
                                focused_edit.cursor_position
                            } else {
                                0
                            };
                            focused_edit.cursor_position = 0;
                            focused_edit.cursor_blink_timer = 0.0;
                        }
                        VirtualKeyCode::End | VirtualKeyCode::Down => {
                            let glyph_len = focused_text.text.graphemes(true).count() as isize;
                            focused_edit.highlight_vector = if modifiers.shift {
                                focused_edit.cursor_position - glyph_len
                            } else {
                                0
                            };
                            focused_edit.cursor_position = glyph_len;
                            focused_edit.cursor_blink_timer = 0.0;
                        }
                        VirtualKeyCode::Back => {
                            if !delete_highlighted(focused_edit, focused_text)
                                && focused_edit.cursor_position > 0
                            {
                                if let Some((byte, len)) = focused_text
                                    .text
                                    .grapheme_indices(true)
                                    .nth(focused_edit.cursor_position as usize - 1)
                                    .map(|i| (i.0, i.1.len()))
                                {
                                    focused_text.text.drain(byte..(byte + len));
                                    focused_edit.cursor_position -= 1;
                                }
                            }
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
                        VirtualKeyCode::Left => {
                            if focused_edit.highlight_vector == 0 || modifiers.shift {
                                if focused_edit.cursor_position > 0 {
                                    let delta = if ctrl_or_cmd(modifiers) {
                                        let mut graphemes = 0;
                                        for word in focused_text.text.split_word_bounds() {
                                            let word_graphemes =
                                                word.graphemes(true).count() as isize;
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
                                focused_edit.cursor_position = focused_edit.cursor_position.min(
                                    focused_edit.cursor_position + focused_edit.highlight_vector,
                                );
                                focused_edit.highlight_vector = 0;
                            }
                        }
                        VirtualKeyCode::Right => {
                            if focused_edit.highlight_vector == 0 || modifiers.shift {
                                let glyph_len = focused_text.text.graphemes(true).count();
                                if (focused_edit.cursor_position as usize) < glyph_len {
                                    let delta = if ctrl_or_cmd(modifiers) {
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
                        }
                        VirtualKeyCode::A => {
                            if ctrl_or_cmd(modifiers) {
                                let glyph_len = focused_text.text.graphemes(true).count() as isize;
                                focused_edit.cursor_position = glyph_len;
                                focused_edit.highlight_vector = -glyph_len;
                            }
                        }
                        VirtualKeyCode::X => {
                            if ctrl_or_cmd(modifiers) {
                                let new_clip = extract_highlighted(focused_edit, focused_text);
                                if !new_clip.is_empty() {
                                    match ClipboardProvider::new().and_then(
                                        |mut ctx: ClipboardContext| ctx.set_contents(new_clip),
                                    ) {
                                        Ok(_) => edit_events.single_write(UiEvent::new(
                                            UiEventType::ValueChange,
                                            entity,
                                        )),
                                        Err(e) => error!(
                                            "Error occured when cutting to clipboard: {:?}",
                                            e
                                        ),
                                    }
                                }
                            }
                        }
                        VirtualKeyCode::C => {
                            if ctrl_or_cmd(modifiers) {
                                let new_clip = read_highlighted(focused_edit, focused_text);
                                if !new_clip.is_empty() {
                                    if let Err(e) = ClipboardProvider::new().and_then(
                                        |mut ctx: ClipboardContext| {
                                            ctx.set_contents(new_clip.to_owned())
                                        },
                                    ) {
                                        error!("Error occured when copying to clipboard: {:?}", e);
                                    }
                                }
                            }
                        }
                        VirtualKeyCode::V => {
                            if ctrl_or_cmd(modifiers) {
                                delete_highlighted(focused_edit, focused_text);

                                match ClipboardProvider::new()
                                    .and_then(|mut ctx: ClipboardContext| ctx.get_contents())
                                {
                                    Ok(contents) => {
                                        let index = cursor_byte_index(focused_edit, focused_text);
                                        let empty_space = focused_edit.max_length
                                            - focused_text.text.graphemes(true).count();
                                        let contents = contents
                                            .graphemes(true)
                                            .take(empty_space)
                                            .fold(String::new(), |mut init, new| {
                                                init.push_str(new);
                                                init
                                            });
                                        focused_text.text.insert_str(index, &contents);
                                        focused_edit.cursor_position +=
                                            contents.graphemes(true).count() as isize;

                                        edit_events.single_write(UiEvent::new(
                                            UiEventType::ValueChange,
                                            entity,
                                        ));
                                    }
                                    Err(e) => error!(
                                        "Error occured when pasting contents of clipboard: {:?}",
                                        e
                                    ),
                                }
                            }
                        }
                        VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                            match focused_text.line_mode {
                                LineMode::Single => {
                                    edit_events.single_write(UiEvent::new(
                                        UiEventType::ValueCommit,
                                        entity,
                                    ));
                                }
                                LineMode::Wrap => {
                                    if modifiers.shift {
                                        if focused_text.text.graphemes(true).count()
                                            < focused_edit.max_length
                                        {
                                            let start_byte = focused_text
                                                .text
                                                .grapheme_indices(true)
                                                .nth(focused_edit.cursor_position as usize)
                                                .map(|i| i.0)
                                                .unwrap_or_else(|| focused_text.text.len());

                                            focused_text.text.insert(start_byte, '\n');
                                            focused_edit.cursor_position += 1;

                                            edit_events.single_write(UiEvent::new(
                                                UiEventType::ValueChange,
                                                entity,
                                            ));
                                        }
                                    } else {
                                        edit_events.single_write(UiEvent::new(
                                            UiEventType::ValueCommit,
                                            entity,
                                        ));
                                    }
                                }
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
fn ctrl_or_cmd(modifiers: ModifiersState) -> bool {
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
    text.text
        .grapheme_indices(true)
        .nth(edit.cursor_position as usize)
        .map(|i| i.0)
        .unwrap_or_else(|| text.text.len())
}

/// Returns the byte indices that are highlighted in the string.
fn highlighted_bytes(edit: &TextEditing, text: &UiText) -> Range<usize> {
    let start = edit
        .cursor_position
        .min(edit.cursor_position + edit.highlight_vector) as usize;
    let end = edit
        .cursor_position
        .max(edit.cursor_position + edit.highlight_vector) as usize;
    let start_byte = text
        .text
        .grapheme_indices(true)
        .nth(start)
        .map(|i| i.0)
        .unwrap_or_else(|| text.text.len());
    let end_byte = text
        .text
        .grapheme_indices(true)
        .nth(end)
        .map(|i| i.0)
        .unwrap_or_else(|| text.text.len());
    start_byte..end_byte
}

fn should_skip_char(input: char) -> bool {
    // Ignore obsolete control characters, and tab characters we can't render
    // properly anyways.  Also ignore newline characters since we don't
    // support multi-line text at the moment.
    input < '\u{20}'
    // Ignore delete character too
    || input == '\u{7F}'
    // Unicode reserves some characters for "private use".  Systems emit
    // these for no clear reason, so we're just going to ignore all of them.
    || (input >= '\u{E000}' && input <= '\u{F8FF}')
    || (input >= '\u{F0000}' && input <= '\u{FFFFF}')
    || (input >= '\u{100000}' && input <= '\u{10FFFF}')
}
