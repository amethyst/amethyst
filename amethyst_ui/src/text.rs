use super::*;
use amethyst_core::shrev::{EventChannel, ReaderId};
use amethyst_core::specs::prelude::{
    Component, DenseVecStorage, Entities, Entity, Join, Read, ReadExpect, ReadStorage, Resources,
    System, Write, WriteStorage,
};
use amethyst_core::timing::Time;
use amethyst_renderer::ScreenDimensions;
use clipboard::{ClipboardContext, ClipboardProvider};
use gfx_glyph::PositionedGlyph;
use hibitset::BitSet;
use std::cmp::Ordering;
use std::ops::Range;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;
use winit::{
    ElementState, Event, KeyboardInput, ModifiersState, MouseButton, VirtualKeyCode, WindowEvent,
};

/// A component used to display text in this entity's UiTransform
#[derive(Clone, Derivative)]
#[derivative(Debug)]
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
    /// Cached glyph positions, used to process mouse highlighting
    #[derivative(Debug = "ignore")]
    pub(crate) cached_glyphs: Vec<PositionedGlyph<'static>>,
    /// Cached `GlyphBrush` id for use in the `UiPass`.
    pub(crate) brush_id: Option<u64>,
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
            cached_glyphs: Vec::new(),
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

impl Component for TextEditing {
    type Storage = DenseVecStorage<Self>;
}

struct CachedTabOrder {
    pub cached: BitSet,
    pub cache: Vec<(i32, Entity)>,
}

/// This system processes the underlying UI data as needed.
pub struct UiKeyboardSystem {
    /// A reader for winit events.
    reader: Option<ReaderId<Event>>,
    /// A cache sorted by tab order, and then by Entity.
    tab_order_cache: CachedTabOrder,
    /// This is set to true while the left mouse button is pressed.
    left_mouse_button_pressed: bool,
    /// The screen coordinates of the mouse
    mouse_position: (f32, f32),
}

impl UiKeyboardSystem {
    /// Creates a new instance of this system
    pub fn new() -> Self {
        Self {
            reader: None,
            tab_order_cache: CachedTabOrder {
                cached: BitSet::new(),
                cache: Vec::new(),
            },
            left_mouse_button_pressed: false,
            mouse_position: (0., 0.),
        }
    }
}

impl<'a> System<'a> for UiKeyboardSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, UiText>,
        WriteStorage<'a, TextEditing>,
        ReadStorage<'a, UiTransform>,
        Write<'a, UiFocused>,
        Read<'a, EventChannel<Event>>,
        Read<'a, Time>,
        ReadExpect<'a, ScreenDimensions>,
    );

    fn run(
        &mut self,
        (entities, mut text, mut editable, transform, mut focused, events, time, screen_dimensions): Self::SystemData,
){
        // Populate and update the tab order cache.
        {
            let bitset = &mut self.tab_order_cache.cached;
            self.tab_order_cache.cache.retain(|&(_t, entity)| {
                let keep = transform.contains(entity);
                if !keep {
                    bitset.remove(entity.id());
                }
                keep
            });
        }

        for &mut (ref mut t, entity) in &mut self.tab_order_cache.cache {
            *t = transform.get(entity).unwrap().tab_order;
        }

        // Attempt to insert the new entities in sorted position.  Should reduce work during
        // the sorting step.
        let transform_set = transform.mask().clone();
        {
            // Create a bitset containing only the new indices.
            let new = (&transform_set ^ &self.tab_order_cache.cached) & &transform_set;
            for (entity, transform, _new) in (&*entities, &transform, &new).join() {
                let pos = self
                    .tab_order_cache
                    .cache
                    .iter()
                    .position(|&(cached_t, _)| transform.tab_order < cached_t);
                match pos {
                    Some(pos) => self
                        .tab_order_cache
                        .cache
                        .insert(pos, (transform.tab_order, entity)),
                    None => self
                        .tab_order_cache
                        .cache
                        .push((transform.tab_order, entity)),
                }
            }
        }
        self.tab_order_cache.cached = transform_set;

        // Sort from smallest tab order to largest tab order, then by entity creation time.
        // Most of the time this shouldn't do anything but you still need it for if the tab orders
        // change.
        self.tab_order_cache
            .cache
            .sort_unstable_by(|&(t1, ref e1), &(t2, ref e2)| {
                let ret = t1.cmp(&t2);
                if ret == Ordering::Equal {
                    return e1.cmp(e2);
                }
                ret
            });

        for text in (&mut text).join() {
            if (*text.text).chars().any(|c| is_combining_mark(c)) {
                let normalized = text.text.nfd().collect::<String>();
                text.text = normalized;
            }
        }

        {
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
        }
        for event in events.read(self.reader.as_mut().unwrap()) {
            // Process events for the whole UI.
            match *event {
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Tab),
                                    modifiers,
                                    ..
                                },
                            ..
                        },
                    ..
                } => if let Some(focused) = focused.entity.as_mut() {
                    if let Some((i, _)) = self
                        .tab_order_cache
                        .cache
                        .iter()
                        .enumerate()
                        .find(|&(_i, &(_, entity))| entity == *focused)
                    {
                        if self.tab_order_cache.cache.len() != 0 {
                            if modifiers.shift {
                                if i == 0 {
                                    let new_i = self.tab_order_cache.cache.len() - 1;
                                    *focused = self.tab_order_cache.cache[new_i].1;
                                } else {
                                    *focused = self.tab_order_cache.cache[i - 1].1;
                                }
                            } else {
                                if i + 1 == self.tab_order_cache.cache.len() {
                                    *focused = self.tab_order_cache.cache[0].1;
                                } else {
                                    *focused = self.tab_order_cache.cache[i + 1].1;
                                }
                            }
                        }
                    }
                },
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    self.mouse_position = (
                        position.x as f32 - screen_dimensions.width() / 2.,
                        position.y as f32 - screen_dimensions.height() / 2.,
                    );
                    if self.left_mouse_button_pressed {
                        let mut focused_text_edit = focused.entity.and_then(|entity| {
                            text.get_mut(entity)
                                .into_iter()
                                .zip(editable.get_mut(entity).into_iter())
                                .next()
                        });
                        if let Some((ref mut focused_text, ref mut focused_edit)) =
                            focused_text_edit
                        {
                            use std::f32::NAN;

                            let mouse_x = self.mouse_position.0 + screen_dimensions.width() / 2.;
                            let mouse_y = self.mouse_position.1 + screen_dimensions.height() / 2.;
                            // Find the glyph closest to the mouse position.
                            focused_edit.highlight_vector = focused_text
                                .cached_glyphs
                                .iter()
                                .enumerate()
                                .fold((0, (NAN, NAN)), |(index, (x, y)), (i, g)| {
                                    let pos = g.position();
                                    // Use Pythagorean theorem to compute distance
                                    if ((x - mouse_x).powi(2) + (y - mouse_y).powi(2)).sqrt()
                                        < ((pos.x - mouse_x).powi(2) + (pos.y - mouse_y).powi(2))
                                            .sqrt()
                                    {
                                        (index, (x, y))
                                    } else {
                                        (i, (pos.x, pos.y))
                                    }
                                })
                                .0
                                as isize
                                - focused_edit.cursor_position;
                            // The end of the text, while not a glyph, is still something
                            // you'll likely want to click your cursor to, so if the cursor is
                            // near the end of the text, check if we should put it at the end
                            // of the text.
                            if focused_edit.cursor_position + focused_edit.highlight_vector + 1
                                == focused_text.cached_glyphs.len() as isize
                            {
                                if let Some(last_glyph) = focused_text.cached_glyphs.iter().last() {
                                    if (last_glyph.position().x - mouse_x).abs()
                                        > ((last_glyph.position().x
                                            + last_glyph.unpositioned().h_metrics().advance_width)
                                            - mouse_x)
                                            .abs()
                                    {
                                        focused_edit.highlight_vector += 1;
                                    }
                                }
                            }
                        }
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
                } => {
                    match state {
                        ElementState::Pressed => {
                            use std::f32::INFINITY;

                            self.left_mouse_button_pressed = true;
                            // Start searching for an element to focus.
                            // Find all eligible elements
                            let mut eligible = (&*entities, &transform)
                                .join()
                                .filter(|&(_, t)| {
                                    t.pixel_x - t.width / 2.0 <= self.mouse_position.0
                                        && t.pixel_x + t.width / 2.0 >= self.mouse_position.0
                                        && t.pixel_y - t.height / 2.0 <= self.mouse_position.1
                                        && t.pixel_y + t.height / 2.0 >= self.mouse_position.1
                                })
                                .collect::<Vec<_>>();
                            // In instances of ambiguity we want to select the element with the
                            // lowest Z order, so we need to find the lowest Z order value among
                            // eligible elements
                            let lowest_z = eligible
                                .iter()
                                .fold(INFINITY, |lowest, &(_, t)| lowest.min(t.global_z));
                            // Then filter by it
                            eligible.retain(|&(_, t)| t.global_z == lowest_z);
                            // We may still have ambiguity as to what to select at this point,
                            // so we'll resolve that by selecting the most recently created
                            // element.
                            focused.entity = eligible.iter().fold(None, |most_recent, &(e, _)| {
                                Some(match most_recent {
                                    Some(most_recent) => if most_recent > e {
                                        most_recent
                                    } else {
                                        e
                                    },
                                    None => e,
                                })
                            });
                            // If we focused an editable text field be sure to position the cursor
                            // in it.
                            let mut focused_text_edit = focused.entity.and_then(|entity| {
                                text.get_mut(entity)
                                    .into_iter()
                                    .zip(editable.get_mut(entity).into_iter())
                                    .next()
                            });
                            if let Some((ref mut focused_text, ref mut focused_edit)) =
                                focused_text_edit
                            {
                                use std::f32::NAN;

                                let mouse_x =
                                    self.mouse_position.0 + screen_dimensions.width() / 2.;
                                let mouse_y =
                                    self.mouse_position.1 + screen_dimensions.height() / 2.;
                                // Find the glyph closest to the click position.
                                focused_edit.highlight_vector = 0;
                                focused_edit.cursor_position = focused_text
                                    .cached_glyphs
                                    .iter()
                                    .enumerate()
                                    .fold((0, (NAN, NAN)), |(index, (x, y)), (i, g)| {
                                        let pos = g.position();
                                        // Use Pythagorean theorem to compute distance
                                        if ((x - mouse_x).powi(2) + (y - mouse_y).powi(2)).sqrt()
                                            < ((pos.x - mouse_x).powi(2)
                                                + (pos.y - mouse_y).powi(2))
                                                .sqrt()
                                        {
                                            (index, (x, y))
                                        } else {
                                            (i, (pos.x, pos.y))
                                        }
                                    })
                                    .0
                                    as isize;
                                // The end of the text, while not a glyph, is still something
                                // you'll likely want to click your cursor to, so if the cursor is
                                // near the end of the text, check if we should put it at the end
                                // of the text.
                                if focused_edit.cursor_position + 1
                                    == focused_text.cached_glyphs.len() as isize
                                {
                                    if let Some(last_glyph) =
                                        focused_text.cached_glyphs.iter().last()
                                    {
                                        if (last_glyph.position().x - mouse_x).abs()
                                            > ((last_glyph.position().x
                                                + last_glyph
                                                    .unpositioned()
                                                    .h_metrics()
                                                    .advance_width)
                                                - mouse_x)
                                                .abs()
                                        {
                                            focused_edit.cursor_position += 1;
                                        }
                                    }
                                }
                            }
                        }
                        ElementState::Released => {
                            self.left_mouse_button_pressed = false;
                        }
                    }
                }
                _ => {}
            }
            let mut focused_text_edit = focused.entity.and_then(|entity| {
                text.get_mut(entity)
                    .into_iter()
                    .zip(editable.get_mut(entity).into_iter())
                    .next()
            });
            // Process events for the focused text element
            if let Some((ref mut focused_text, ref mut focused_edit)) = focused_text_edit {
                match *event {
                    Event::WindowEvent {
                        event: WindowEvent::ReceivedCharacter(input),
                        ..
                    } => {
                        // Ignore obsolete control characters, and tab characters we can't render
                        // properly anyways.  Also ignore newline characters since we don't
                        // support multi-line text at the moment.
                        if input < '\u{20}' {
                            continue;
                        }
                        // Ignore delete character too
                        else if input == '\u{7F}' {
                            continue;
                        }
                        // Unicode reserves some characters for "private use".  Systems emit
                        // these for no clear reason, so we're just going to ignore all of them.
                        else if input >= '\u{E000}' && input <= '\u{F8FF}' {
                            continue;
                        } else if input >= '\u{F0000}' && input <= '\u{FFFFF}' {
                            continue;
                        } else if input >= '\u{100000}' && input <= '\u{10FFFF}' {
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
                            if !delete_highlighted(focused_edit, focused_text) {
                                if focused_edit.cursor_position > 0 {
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
                        }
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
                                let empty_space = focused_edit.max_length
                                    - focused_text.text.graphemes(true).count();
                                let contents = contents.graphemes(true).take(empty_space).fold(
                                    String::new(),
                                    |mut init, new| {
                                        init.push_str(new);
                                        init
                                    },
                                );
                                focused_text.text.insert_str(index, &contents);
                                focused_edit.cursor_position +=
                                    contents.graphemes(true).count() as isize;
                            }
                        },
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use amethyst_core::specs::prelude::SystemData;
        Self::SystemData::setup(res);
        self.reader = Some(res.fetch_mut::<EventChannel<Event>>().register_reader());
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
    text.text
        .grapheme_indices(true)
        .nth(edit.cursor_position as usize)
        .map(|i| i.0)
        .unwrap_or(text.text.len())
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
        .unwrap_or(text.text.len());
    let end_byte = text
        .text
        .grapheme_indices(true)
        .nth(end)
        .map(|i| i.0)
        .unwrap_or(text.text.len());
    start_byte..end_byte
}
