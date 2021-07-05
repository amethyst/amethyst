//! Crate abstracting and separating out the window and display handling within amethyst, and as such
//! its usage of winit.

#![doc(
    html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
    html_root_url = "https://docs.amethyst.rs/stable"
)]
#![deny(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility,
    clippy::all
)]
#![warn(clippy::pedantic)]
#![allow(clippy::new_without_default, clippy::module_name_repetitions)]

mod bundle;
mod config;
mod monitor;
mod resources;
mod system;

pub use winit::window::Window;

#[cfg(feature = "test-support")]
pub use crate::bundle::{SCREEN_HEIGHT, SCREEN_WIDTH};
pub use crate::{
    bundle::WindowBundle,
    config::DisplayConfig,
    monitor::{MonitorIdent, MonitorsAccess},
    resources::ScreenDimensions,
    system::*,
};
