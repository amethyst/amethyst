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

pub(crate) use amethyst_core::ecs::Entity;

pub use self::{
    blink::*,
    bundle::UiBundle,
    button::{
        UiButton, UiButtonAction, UiButtonActionRetrigger, UiButtonActionRetriggerSystem,
        UiButtonActionRetriggerSystemDesc, UiButtonActionType, UiButtonBuilder,
        UiButtonBuilderResources, UiButtonSystem, UiButtonSystemDesc,
    },
    drag::{DragWidgetSystemDesc, Draggable},
    event::{
        targeted, targeted_below, Interactable, TargetedEvent, UiEvent, UiEventType, UiMouseSystem,
    },
    event_retrigger::{
        EventReceiver, EventRetrigger, EventRetriggerSystem, EventRetriggerSystemDesc,
    },
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, FontHandle, TtfFormat},
    glyphs::{UiGlyphsSystem, UiGlyphsSystemDesc},
    image::UiImage,
    label::{UiLabel, UiLabelBuilder, UiLabelBuilderResources},
    layout::{Anchor, ScaleMode, Stretch, UiTransformSystem, UiTransformSystemDesc},
    pass::{DrawUi, DrawUiDesc, RenderUi},
    prefab::{
        NoCustomUi, TextEditingPrefab, ToNativeWidget, UiButtonData, UiCreator, UiFormat,
        UiImageLoadPrefab, UiImagePrefab, UiLoader, UiLoaderSystem, UiLoaderSystemDesc, UiPrefab,
        UiTextData, UiTransformData, UiWidget,
    },
    resize::{ResizeSystem, ResizeSystemDesc, UiResize},
    selection::{
        Selectable, Selected, SelectionKeyboardSystem, SelectionKeyboardSystemDesc,
        SelectionMouseSystem, SelectionMouseSystemDesc,
    },
    selection_order_cache::{CacheSelectionOrderSystem, CachedSelectionOrder},
    sound::{
        UiPlaySoundAction, UiSoundRetrigger, UiSoundRetriggerSystem, UiSoundRetriggerSystemDesc,
        UiSoundSystem,
    },
    text::{LineMode, TextEditing, TextEditingMouseSystem, TextEditingMouseSystemDesc, UiText},
    text_editing::{TextEditingInputSystem, TextEditingInputSystemDesc},
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
mod prefab;
mod resize;
mod selection;
mod selection_order_cache;
mod sound;
mod text;
mod text_editing;
mod transform;
mod widgets;
