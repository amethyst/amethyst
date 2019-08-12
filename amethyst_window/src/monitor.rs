use serde::{Deserialize, Serialize};
use winit::{monitor::MonitorHandle, event_loop::EventLoop};

/// Identifier for a given monitor. Because there is no cross platform method to actually uniquely
/// identify monitors, this tuple wraps two identifiers of a monitor which should prove sufficient
/// on any given system: The index of the monitor is the retrieved Display array, and the name
/// of the given monitor.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg(not(target_arch = "wasm32"))]
pub struct MonitorIdent {
    #[cfg(target_os = "windows")]
    name: String,
    #[cfg(not(target_os = "windows"))]
    name: Option<String>,
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "macos",
    ))]
    native_id: u32,
}

/// Identifier for a given monitor. Because there is no cross platform method to actually uniquely
/// identify monitors, this tuple wraps two identifiers of a monitor which should prove sufficient
/// on any given system: The index of the monitor is the retrieved Display array, and the name
/// of the given monitor.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg(target_arch = "wasm32")]
pub struct MonitorIdent;

/// A semi-stable serializable identifier for a monitor.
/// Useful for keeping fullscreen configuration persistent.
impl MonitorIdent {
    /// Get the identifier for current primary monitor.
    pub fn from_primary<T: 'static>(event_loop: &EventLoop<T>) -> Self {
        Self::from_monitor(&event_loop.primary_monitor())
    }

    /// Get the identifier for specific monitor id.    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_monitor(monitor: &MonitorHandle) -> Self {
        MonitorIdent {
            #[cfg(target_os = "windows")]
            name: monitor.name().unwrap(),
            #[cfg(not(target_os = "windows"))]
            name: monitor.name(),
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            native_id: winit::platform::unix::MonitorHandleExtUnix::native_id(&monitor),
            #[cfg(target_os = "macos")]
            native_id: winit::platform::macos::MonitorHandleExtMacOS::native_id(&monitor),
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn from_monitor(_monitor: &MonitorHandle) -> Self {
        MonitorIdent
    }

    /// Select a monitor that matches this identifier most closely.
    pub fn monitor<T: 'static>(&self, event_loop: &EventLoop<T>) -> MonitorHandle {
        event_loop.available_monitors()
            .find(|m| Self::from_monitor(m) == *self).unwrap()
    }
}
