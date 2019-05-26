mod bundle;
mod config;
mod monitor;
mod resources;
mod system;

#[cfg(feature = "test-support")]
pub use crate::bundle::{SCREEN_HEIGHT, SCREEN_WIDTH};
pub use crate::{
    bundle::WindowBundle,
    config::DisplayConfig,
    monitor::{MonitorIdent, MonitorsAccess},
    resources::ScreenDimensions,
    system::{EventsLoopSystem, WindowSystem},
};
pub use winit::{Icon, Window};
