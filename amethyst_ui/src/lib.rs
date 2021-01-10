//! Provides components and systems to create an in game user interface.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub use self::{
    blink::*,
    bundle::{AudioUiBundle, UiBundle},
    button::{
        UiButton, UiButtonAction, UiButtonActionRetrigger, UiButtonActionType, UiButtonBuilder,
    },
    drag::{DragWidgetSystem, Draggable},
    event::{targeted, targeted_below, Interactable, TargetedEvent, UiEvent, UiEventType},
    event_retrigger::{EventReceiver, EventRetrigger},
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, TtfFormat},
    glyphs::UiGlyphsSystem,
    image::UiImage,
    label::{UiLabel, UiLabelBuilder},
    layout::{Anchor, ScaleMode, Stretch},
    pass::{DrawUi, DrawUiDesc, RenderUi},
    resize::{ResizeSystem, UiResize},
    selection::{Selectable, Selected, SelectionKeyboardSystem, SelectionMouseSystem},
    selection_order_cache::{CacheSelectionSystem, CachedSelectionOrderResource},
    sound::{UiPlaySoundAction, UiSoundRetrigger, UiSoundSystem},
    text::{LineMode, TextEditing, TextEditingMouseSystem, UiText},
    text_editing::TextEditingInputSystem,
    transform::{get_parent_pixel_size, UiFinder, UiTransform},
    widgets::{Widget, WidgetId, Widgets},
};

mod blink;
mod bundle;
mod button;
mod drag;
mod event;
mod event_retrigger;
mod font;
mod format;
mod glyphs;
mod image;
mod label;
mod layout;
mod pass;
mod resize;
mod selection;
mod selection_order_cache;
mod sound;
mod text;
mod text_editing;
mod transform;
mod widgets;
