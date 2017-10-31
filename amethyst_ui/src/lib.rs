//! Provides components and systems to create an in game user interface.

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate amethyst_assets;
extern crate amethyst_core;
extern crate amethyst_renderer;
extern crate cgmath;
extern crate gfx;
extern crate hibitset;
extern crate rayon;
extern crate rusttype;
extern crate shrev;
extern crate specs;
extern crate unicode_normalization;
extern crate winit;

mod bundle;
mod format;
mod image;
mod pass;
mod resize;
mod text;
mod transform;

pub use self::bundle::UiBundle;
pub use self::format::{FontAsset, FontHandle, OtfFormat, TtfFormat};
pub use self::image::UiImage;
pub use self::pass::DrawUi;
pub use self::resize::{UiResize, ResizeSystem};
pub use self::text::{UiText, UiTextRenderer};
pub use self::transform::UiTransform;
