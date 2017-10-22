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
extern crate specs;

mod bundle;
mod image;
mod pass;
mod transform;

pub use self::bundle::UiBundle;
pub use self::image::UiImage;
pub use self::pass::DrawUi;
pub use self::transform::UiTransform;
