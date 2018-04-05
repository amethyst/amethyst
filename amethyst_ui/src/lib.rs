//! Provides components and systems to create an in game user interface.

#![warn(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_input;
extern crate amethyst_renderer;
extern crate clipboard;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate gfx;
extern crate gfx_glyph;
extern crate hibitset;
extern crate rusttype;
extern crate shrev;
#[macro_use]
extern crate shred_derive;
extern crate shred;
extern crate specs;
extern crate unicode_normalization;
extern crate unicode_segmentation;
extern crate winit;

#[macro_use]
#[cfg(feature = "profiler")]
extern crate thread_profiler;

mod bundle;
mod button;
mod event;
mod focused;
mod format;
mod image;
mod layout;
mod pass;
mod resize;
mod text;
mod transform;

pub use self::bundle::UiBundle;
pub use self::button::{UiButton, UiButtonBuilder, UiButtonResources};
pub use self::event::{MouseReactive, UiEvent, UiEventType, UiMouseSystem};
pub use self::focused::UiFocused;
pub use self::format::{FontAsset, FontHandle, OtfFormat, TtfFormat};
pub use self::image::UiImage;
pub use self::layout::{Anchor, Anchored, ScaleMode, Stretch, Stretched, UiLayoutSystem,
                       UiParentSystem};
pub use self::pass::DrawUi;
pub use self::resize::{ResizeSystem, UiResize};
pub use self::text::{TextEditing, UiSystem, UiText};
pub use self::transform::UiTransform;

/// How many times the cursor blinks per second while editing text.
const CURSOR_BLINK_RATE: f32 = 2.0;
