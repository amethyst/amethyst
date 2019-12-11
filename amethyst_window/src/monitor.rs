use serde::{Deserialize, Serialize};
use winit::{event_loop::EventLoop, monitor::MonitorHandle, window::Window};

/// A struct that can resolve monitors.
/// Usually either a Window or an EventsLoop.
pub trait MonitorsAccess {
    /// Returns an iterator over the available monitors
    fn iter(&self) -> Box<Iterator<Item = MonitorHandle>>;
    /// Returns the `MonitorHandle` of the primary display
    fn primary(&self) -> MonitorHandle;
}

impl MonitorsAccess for EventLoop<()> {
    fn iter(&self) -> Box<Iterator<Item = MonitorHandle>> {
        Box::new(self.available_monitors())
    }
    fn primary(&self) -> MonitorHandle {
        self.primary_monitor()
    }
}

impl MonitorsAccess for Window {
    fn iter(&self) -> Box<Iterator<Item = MonitorHandle>> {
        Box::new(self.available_monitors())
    }
    fn primary(&self) -> MonitorHandle {
        self.primary_monitor()
    }
}

/// Identifier for a given monitor. Because there is no cross platform method to actually uniquely
/// identify monitors, this tuple wraps two identifiers of a monitor which should prove sufficient
/// on any given system: The index of the monitor is the retrieved Display array, and the name
/// of the given monitor.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MonitorIdent(u16, String);

/// A semi-stable serializable identifier for a monitor.
/// Useful for keeping fullscreen configuration persistent.
impl MonitorIdent {
    /// Get the identifier for current primary monitor.
    pub fn from_primary(monitors: &impl MonitorsAccess) -> Self {
        Self::from_monitor_id(monitors, monitors.primary())
            .expect("Primary monitor not found in the list of all monitors")
    }

    /// Get the identifier for specific monitor id.
    pub fn from_monitor_id(
        _monitors: &impl MonitorsAccess,
        _monitor: MonitorHandle,
    ) -> Option<Self> {
        // @todo reimplement serializable monitor identification
        // #[cfg(target_os = "ios")]
        // use winit::platform::windows::MonitorIdExt;
        // #[cfg(target_os = "macos")]
        // use winit::platform::macos::MonitorIdExt;
        // #[cfg(any(
        //     target_os = "linux",
        //     target_os = "dragonfly",
        //     target_os = "freebsd",
        //     target_os = "netbsd",
        //     target_os = "openbsd"
        // ))]
        // use winit::platform::unix::MonitorHandleExtUnix;
        // #[cfg(target_os = "windows")]
        // use winit::platform::windows::MonitorIdExt;

        // let native_id = monitor_id.native_id();
        // monitors
        //     .iter()
        //     .enumerate()
        //     .find(|(_, m)| m.native_id() == native_id)
        //     .and_then(|(i, m)| m.get_name().map(|name| Self(i as u16, name)))
        None
    }

    /// Select a monitor that matches this identifier most closely.
    pub fn monitor_id(&self, monitors: &impl MonitorsAccess) -> MonitorHandle {
        monitors
            .iter()
            .enumerate()
            .filter(|(_, m)| m.name().map(|n| n == self.1).unwrap_or(false))
            .max_by_key(|(i, _)| (*i as i32 - i32::from(self.0)).abs() as u16)
            .map(|(_, m)| m)
            .unwrap_or_else(|| monitors.primary())
    }
}
