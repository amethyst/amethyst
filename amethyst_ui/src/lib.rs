//! Provides components and systems to create an in game user interface.

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub use self::{
    blink::BlinkSystem,
    bundle::UiBundle,
    button::{
        UiButton, UiButtonAction, UiButtonActionRetrigger, UiButtonActionRetriggerSystem,
        UiButtonActionType, UiButtonBuilder, UiButtonBuilderResources, UiButtonSystem,
    },
    event::{targeted, Interactable, UiEvent, UiEventType, UiMouseSystem},
    event_retrigger::{EventReceiver, EventRetriggerSystem},
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, FontHandle, TtfFormat},
    glyphs::UiGlyphsSystem,
    image::UiImage,
    label::{UiLabel, UiLabelBuilder, UiLabelBuilderResources},
    layout::{Anchor, ScaleMode, Stretch, UiTransformSystem},
    pass::{DrawUi, DrawUiDesc, RenderUi},
    prefab::{
        NoCustomUi, ToNativeWidget, UiCreator, UiFormat, UiImagePrefab, UiLoader, UiLoaderSystem,
        UiPrefab, UiTextBuilder, UiTransformBuilder, UiWidget,
    },
    resize::{ResizeSystem, UiResize},
    selection::{Selectable, Selected, SelectionKeyboardSystem, SelectionMouseSystem},
    selection_order_cache::{CacheSelectionOrderSystem, CachedSelectionOrder},
    sound::{UiPlaySoundAction, UiSoundRetrigger, UiSoundRetriggerSystem, UiSoundSystem},
    text::{LineMode, TextEditing, TextEditingMouseSystem, UiText},
    text_editing::TextEditingInputSystem,
    transform::{UiFinder, UiTransform},
    widgets::{Widget, WidgetId, Widgets},
};

pub(crate) use amethyst_core::ecs::prelude::Entity;
pub(crate) use paste;

mod blink;
mod bundle;
mod button;
mod event;
mod event_retrigger;
mod font;
mod format;
mod glyphs;
mod image;
mod label;
mod layout;
mod pass;
mod prefab;
mod resize;
mod selection;
mod selection_order_cache;
mod sound;
mod text;
mod text_editing;
mod transform;
mod widgets;
