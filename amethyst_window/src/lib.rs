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
mod config;
mod monitor;
mod resources;
mod system;

#[cfg(feature = "test-support")]
#[doc(no_inline)]
pub use crate::bundle::{SCREEN_HEIGHT, SCREEN_WIDTH};
#[doc(no_inline)]
pub use crate::{
    bundle::WindowBundle,
    config::DisplayConfig,
    monitor::{MonitorIdent, MonitorsAccess},
    resources::ScreenDimensions,
    system::{EventsLoopSystem, WindowSystem},
};
#[doc(no_inline)]
pub use winit::{Icon, Window};
