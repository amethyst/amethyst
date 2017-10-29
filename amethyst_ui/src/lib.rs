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
extern crate specs;
extern crate unicode_normalization;

mod bundle;
mod format;
mod image;
mod pass;
mod text;
mod transform;

pub use self::bundle::UiBundle;
pub use self::format::{FontFileAsset, FontFormat};
pub use self::image::UiImage;
pub use self::pass::DrawUi;
pub use self::text::{UiText, UiTextRenderer};
pub use self::transform::UiTransform;
