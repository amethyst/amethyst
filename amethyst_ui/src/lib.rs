//! Provides components and systems to create an in game user interface.

#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

pub use self::{
    blink::BlinkSystem,
    bundle::UiBundle,
    button::{
        UiButton, UiButtonAction, UiButtonActionRetriggerComponent, UiButtonActionRetriggerSystem,
        UiButtonActionType, UiButtonBuilder, UiButtonBuilderResources, UiButtonSystem,
    },
    event::{targeted, InteractableComponent, UiEvent, UiEventType, UiMouseSystem},
    event_retrigger::{EventReceiver, EventRetriggerSystem},
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, FontHandle, TtfFormat},
    glyphs::UiGlyphsSystem,
    image::UiImageComponent,
    label::{UiLabel, UiLabelBuilder, UiLabelBuilderResources},
    layout::{Anchor, ScaleMode, Stretch, UiTransformSystem},
    pass::{DrawUi, DrawUiDesc},
    prefab::{
        NoCustomUi, ToNativeWidget, UiCreator, UiFormat, UiImagePrefab, UiLoader, UiLoaderSystem,
        UiPrefab, UiTextBuilder, UiTransformBuilder, UiWidget,
    },
    resize::{ResizeSystem, UiResizeComponent},
    selection::{Selectable, SelectedComponent, SelectionKeyboardSystem, SelectionMouseSystem},
    selection_order_cache::{CacheSelectionOrderSystem, CachedSelectionOrder},
    sound::{UiPlaySoundAction, UiSoundRetriggerComponent, UiSoundRetriggerSystem, UiSoundSystem},
    text::{LineMode, TextEditingComponent, TextEditingMouseSystem, UiTextComponent},
    text_editing::TextEditingInputSystem,
    transform::{UiFinder, UiTransformComponent},
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
