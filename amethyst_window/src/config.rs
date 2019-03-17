use serde::{Deserialize, Serialize};
use winit::{WindowAttributes, WindowBuilder};

use crate::monitor::{MonitorIdent, MonitorsAccess};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DisplayConfig {
    /// Name of the application window.
    #[serde(default = "default_title")]
    pub title: String,
    /// Enables fullscreen mode on specific monitor when set.
    /// Defaults to `None`, which means fullscreen is off.
    #[serde(default)]
    pub fullscreen: Option<MonitorIdent>,
    /// Current window dimensions, measured in pixels (px).
    #[serde(default)]
    pub dimensions: Option<(u32, u32)>,
    /// Minimum window dimensions, measured in pixels (px).
    #[serde(default)]
    pub min_dimensions: Option<(u32, u32)>,
    /// Maximum window dimensions, measured in pixels (px).
    #[serde(default)]
    pub max_dimensions: Option<(u32, u32)>,
    #[serde(default = "default_visibility")]
    pub visibility: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            title: default_title(),
            fullscreen: None,
            dimensions: None,
            min_dimensions: None,
            max_dimensions: None,
            visibility: default_visibility(),
        }
    }
}

fn default_title() -> String {
    "Amethyst game".to_string()
}
fn default_visibility() -> bool {
    true
}

impl DisplayConfig {
    /// Creates a `winit::WindowBuilder` using the values set in the DisplayConfig
    ///
    /// The `MonitorsAccess` is needed to configure a fullscreen window.
    pub fn to_windowbuilder(self, monitors: &impl MonitorsAccess) -> WindowBuilder {
        let attrs = WindowAttributes {
            dimensions: self.dimensions.map(Into::into),
            max_dimensions: self.max_dimensions.map(Into::into),
            min_dimensions: self.min_dimensions.map(Into::into),
            title: self.title,
            visible: self.visibility,
            fullscreen: self.fullscreen.map(|ident| ident.monitor_id(monitors)),
            ..Default::default()
        };

        let mut builder = WindowBuilder::new();
        builder.window = attrs;
        builder
    }
}
