use serde::{Deserialize, Serialize};
use winit::{AvailableMonitorsIter, EventsLoop, MonitorId, Window};

/// A struct that can resolve monitors.
/// Usually either a Window or an EventsLoop.
pub trait MonitorsAccess {
    fn iter(&self) -> AvailableMonitorsIter;
    fn primary(&self) -> MonitorId;
}

impl MonitorsAccess for EventsLoop {
    fn iter(&self) -> AvailableMonitorsIter {
        self.get_available_monitors()
    }
    fn primary(&self) -> MonitorId {
        self.get_primary_monitor()
    }
}

impl MonitorsAccess for Window {
    fn iter(&self) -> AvailableMonitorsIter {
        self.get_available_monitors()
    }
    fn primary(&self) -> MonitorId {
        self.get_primary_monitor()
    }
}

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
    pub fn from_monitor_id(monitors: &impl MonitorsAccess, monitor_id: MonitorId) -> Option<Self> {
        #[cfg(target_os = "ios")]
        use winit::ios::windows::MonitorIdExt;
        #[cfg(target_os = "macos")]
        use winit::os::macos::MonitorIdExt;
        #[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
        use winit::os::unix::MonitorIdExt;
        #[cfg(target_os = "windows")]
        use winit::os::windows::MonitorIdExt;

        let native_id = monitor_id.native_id();
        monitors
            .iter()
            .enumerate()
            .find(|(_, m)| m.native_id() == native_id)
            .and_then(|(i, m)| m.get_name().map(|name| Self(i as u16, name)))
    }

    /// Select a monitor that matches this identifier most closely.
    pub fn monitor_id(&self, monitors: &impl MonitorsAccess) -> MonitorId {
        monitors
            .iter()
            .enumerate()
            .filter(|(_, m)| m.get_name().map(|n| n == self.1).unwrap_or(false))
            .max_by_key(|(i, _)| (*i as i32 - self.0 as i32).abs() as u16)
            .map(|(_, m)| m)
            .unwrap_or_else(|| monitors.primary())
    }
}
