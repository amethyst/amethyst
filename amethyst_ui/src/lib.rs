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
    bundle::UiBundle,
    button::{
        UiButton, UiButtonAction, UiButtonActionRetrigger, build_button_action_retrigger_system,
        UiButtonActionType, UiButtonBuilder, build_ui_button_system,
    },
    drag::{Draggable,build_drag_widget_system},
    event::{
        targeted, targeted_below, Interactable, TargetedEvent, UiEvent, UiEventType, build_ui_mouse_system,
    },
    event_retrigger::{
        EventReceiver, EventRetrigger, build_event_retrigger_system,
    },
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, FontHandle, TtfFormat},
    glyphs::build_ui_glyphs_system,
    image::UiImage,
    label::{UiLabel, UiLabelBuilder},
    layout::{Anchor, ScaleMode, Stretch, build_ui_transform_system},
    pass::{DrawUi, DrawUiDesc, RenderUi},
    /*
    prefab::{
        NoCustomUi, TextEditingPrefab, ToNativeWidget, UiButtonData, UiCreator, UiFormat,
        UiImageLoadPrefab, UiImagePrefab, UiLoader, UiLoaderSystem, UiLoaderSystemDesc, UiPrefab,
        UiTextData, UiTransformData, UiWidget,
    },
    */
    resize::{build_resize_system, UiResize},
    selection::{
        Selectable, Selected, build_selection_keyboard_system, build_selection_mouse_system,
    },
    selection_order_cache::{build_cache_selection_system, CachedSelectionOrderResource},
    sound::{
        UiPlaySoundAction, UiSoundRetrigger, build_ui_sound_retrigger_system,
        build_ui_sound_system,
    },
    text::{LineMode, TextEditing, build_text_editing_mouse_system, UiText},
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
