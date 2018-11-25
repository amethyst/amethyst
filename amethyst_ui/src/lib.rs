//! Provides components and systems to create an in game user interface.

#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

use amethyst_assets;
use amethyst_audio;
use amethyst_core;

use amethyst_renderer;
use clipboard;
#[macro_use]
extern crate derivative;
use fnv;

use gfx;
use gfx_glyph;
use glsl_layout;
use hibitset;
#[macro_use]
extern crate log;
use ron;
#[macro_use]
extern crate serde;
use shred;
#[macro_use]
extern crate shred_derive;

use winit;
#[macro_use]
extern crate smallvec;
extern crate unicode_normalization;
extern crate unicode_segmentation;

mod bundle;
mod button;
mod event;
mod event_retrigger;
mod focused;
mod font;
mod format;
mod image;
mod layout;
mod pass;
mod prefab;
mod resize;
mod sound;
mod text;
mod transform;

pub use self::{
    bundle::UiBundle,
    button::{
        UiButton, UiButtonAction, UiButtonActionRetrigger, UiButtonActionRetriggerSystem,
        UiButtonActionType, UiButtonBuilder, UiButtonBuilderResources, UiButtonSystem,
    },
    event::{MouseReactive, UiEvent, UiEventType, UiMouseSystem},
    event_retrigger::{EventReceiver, EventRetriggerSystem},
    focused::UiFocused,
    font::{
        default::get_default_font,
        systemfont::{default_system_font, get_all_font_handles, list_system_font_families},
    },
    format::{FontAsset, FontFormat, FontHandle, OtfFormat, TtfFormat},
    image::UiImage,
    layout::{Anchor, ScaleMode, Stretch, UiTransformSystem},
    pass::DrawUi,
    prefab::{
        NoCustomUi, ToNativeWidget, UiCreator, UiFormat, UiImageBuilder, UiLoader, UiLoaderSystem,
        UiPrefab, UiTextBuilder, UiTransformBuilder, UiWidget,
    },
    resize::{ResizeSystem, UiResize},
    sound::{UiPlaySoundAction, UiSoundRetrigger, UiSoundRetriggerSystem, UiSoundSystem},
    text::{LineMode, TextEditing, UiKeyboardSystem, UiText},
    transform::{UiFinder, UiTransform},
};

/// How many times the cursor blinks per second while editing text.
const CURSOR_BLINK_RATE: f32 = 2.0;
