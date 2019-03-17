mod config;
mod monitor;
mod resources;
mod system;

pub use crate::{
    config::DisplayConfig,
    monitor::{MonitorIdent, MonitorsAccess},
    resources::ScreenDimensions,
    system::{EventsLoopSystem, WindowSystem},
};
