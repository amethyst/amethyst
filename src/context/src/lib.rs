#![crate_name = "amethyst_context"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

//! A way to manage engine resources
//!
//! # Basic usage:

//! ```rust
//! extern crate amethyst_config;
//! extern crate amethyst_context;
//!
//! use amethyst_config::Element;
//! use amethyst_context::{Context, Config};
//!
//!
//! fn main() {
//!     let config = Config::default();
//!     let context = Context::new(config);
//!     // Now resources provided by Context are available
//! }
//! ```
//! See `amethyst/examples/window.rs` for an example.

#[macro_use]
extern crate amethyst_config;
extern crate glutin;

use amethyst_config::Element;
use std::path::Path;

mod video_context;
pub use video_context::{VideoContext, DisplayConfig};

config!(
    /// Contains configs for resources provided by `Context`
    struct Config {
    pub display_config: DisplayConfig = DisplayConfig::default(),
});

/// Contains all engine resources which are shared by `State`s, in particular `Window` and `VideoContext`
pub struct Context {
    pub video_context: VideoContext,
}

impl Context {
    //! Creates a `Context` configured according to `Config`
    pub fn new(config: Config) -> Option<Context> {
        let video_context =
            match VideoContext::new(config.display_config) {
                Some(video_context) => video_context,
                None => return None,
            };

        Some(
            Context {
                video_context: video_context,
            }
        )
    }
}
