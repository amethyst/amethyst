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
        build_button_action_retrigger_system, build_ui_button_system, UiButton, UiButtonAction,
        UiButtonActionRetrigger, UiButtonActionType, UiButtonBuilder,
    },
    drag::{build_drag_widget_system, Draggable},
    event::{
        build_ui_mouse_system, targeted, targeted_below, Interactable, TargetedEvent, UiEvent,
        UiEventType,
    },
    event_retrigger::{build_event_retrigger_system, EventReceiver, EventRetrigger},
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, FontHandle, TtfFormat},
    glyphs::build_ui_glyphs_system,
    image::UiImage,
    label::{UiLabel, UiLabelBuilder},
    layout::{build_ui_transform_system, Anchor, ScaleMode, Stretch},
    pass::{DrawUi, DrawUiDesc, RenderUi},
    resize::{build_resize_system, UiResize},
    selection::{
        build_selection_keyboard_system, build_selection_mouse_system, Selectable, Selected,
    },
    selection_order_cache::{build_cache_selection_system, CachedSelectionOrderResource},
    sound::{
        build_ui_sound_retrigger_system, build_ui_sound_system, UiPlaySoundAction, UiSoundRetrigger,
    },
    text::{build_text_editing_mouse_system, LineMode, TextEditing, UiText},
    text_editing::build_text_editing_input_system,
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
//mod prefab;
mod resize;
mod selection;
mod selection_order_cache;
mod sound;
mod text;
mod text_editing;
mod transform;
mod widgets;
