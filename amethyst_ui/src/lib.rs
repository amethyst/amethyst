//! Provides components and systems to create an in game user interface.

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate amethyst_assets;
extern crate amethyst_renderer;
extern crate specs;

mod image;
mod transform;

pub use self::image::UiImage;
pub use self::transform::UiTransform;
