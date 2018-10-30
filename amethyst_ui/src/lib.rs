//! Provides components and systems to create an in game user interface.

#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(type_complexity))] // complex project

extern crate amethyst_assets;
extern crate amethyst_audio;
extern crate amethyst_core;
extern crate amethyst_input;
extern crate amethyst_renderer;
extern crate clipboard;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate font_kit;
extern crate gfx;
extern crate gfx_glyph;
#[macro_use]
extern crate glsl_layout;
extern crate hibitset;
#[macro_use]
extern crate log;
extern crate ron;
#[macro_use]
extern crate serde;
extern crate shred;
#[macro_use]
extern crate shred_derive;
extern crate unicode_normalization;
extern crate unicode_segmentation;
extern crate winit;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

mod action_components;
mod bundle;
mod button;
mod event;
mod focused;
mod font;
mod format;
mod image;
mod layout;
mod pass;
mod prefab;
mod resize;
mod text;
mod transform;

pub use self::{
    action_components::{OnUiActionImage, OnUiActionSound},
    bundle::UiBundle,
    button::{UiButton, UiButtonBuilder, UiButtonBuilderResources, UiButtonSystem},
    event::{MouseReactive, UiEvent, UiEventType, UiMouseSystem},
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
        UiCreator, UiFormat, UiImageBuilder, UiLoader, UiLoaderSystem, UiPrefab, UiTextBuilder,
        UiTransformBuilder, UiWidget,
    },
    resize::{ResizeSystem, UiResize},
    text::{LineMode, TextEditing, UiKeyboardSystem, UiText},
    transform::{UiFinder, UiTransform},
};

/// How many times the cursor blinks per second while editing text.
const CURSOR_BLINK_RATE: f32 = 2.0;
