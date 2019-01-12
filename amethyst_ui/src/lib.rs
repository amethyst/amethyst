//! Provides components and systems to create an in game user interface.

#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate shred_derive;

#[macro_use]
extern crate smallvec;

pub use self::{
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
    format::{FontAsset, FontFormat, FontHandle, OtfFormat, TtfFormat},
    layout::{Anchor, ScaleMode, Stretch, UiTransformSystem},
    pass::DrawUi,
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
};

use clipboard;
use fnv;
use gfx;
use gfx_glyph;
use glsl_layout;
use hibitset;
use ron;
use shred;
use unicode_normalization;
use unicode_segmentation;
use winit;

use amethyst_assets;
use amethyst_audio;
use amethyst_core;
use amethyst_renderer;

mod bundle;
mod button;
mod event;
mod event_retrigger;
mod font;
mod format;
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
