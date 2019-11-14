//! Crate abstracting and seperating out the window and display handling within amethyst, and as such
//! its usage of winit.

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

mod bundle;
mod display_config;
mod monitor;
mod resources;
mod system;

#[cfg(feature = "test-support")]
pub use crate::bundle::{SCREEN_HEIGHT, SCREEN_WIDTH};
pub use crate::{
    bundle::WindowBundle,
    display_config::DisplayConfig,
    monitor::{MonitorIdent, MonitorsAccess},
    resources::ScreenDimensions,
    system::WindowSystem,
};
pub use winit::{event_loop::EventLoop, window::{Icon, Window}};
