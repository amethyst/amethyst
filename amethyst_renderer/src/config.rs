//! Renderer configuration.

use serde::{Deserialize, Serialize};
use winit::{self, dpi::LogicalSize, WindowBuilder};

/// Structure for holding the renderer configuration.
///
/// # Examples
///
/// Example Ron config file:
/// ```ron
/// (
///     title: "Game title",
///     dimensions: Some((640, 480)),
///     max_dimensions: None,
///     min_dimensions: None,
///     fullscreen: false,
///     multisampling: 0,
///     visibility: true,
///     vsync: true
/// )
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DisplayConfig {
    /// Name of the application window.
    #[serde(default = "default_title")]
    pub title: String,
    /// Enables or disables fullscreen mode.
    #[serde(default)]
    pub fullscreen: bool,
    /// Current window dimensions, measured in pixels (px).
    #[serde(default)]
    pub dimensions: Option<(u32, u32)>,
    /// Minimum window dimensions, measured in pixels (px).
    #[serde(default)]
    pub min_dimensions: Option<(u32, u32)>,
    /// Maximum window dimensions, measured in pixels (px).
    #[serde(default)]
    pub max_dimensions: Option<(u32, u32)>,
    /// Enables or disables vertical synchronization.
    #[serde(default = "default_vsync")]
    pub vsync: bool,
    /// Level of MSAA anti-aliasing.
    #[serde(default = "default_multisampling")]
    pub multisampling: u16,
    /// Sets the visibility of the window.
    #[serde(default = "default_visibility")]
    pub visibility: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            title: default_title(),
            fullscreen: false,
            dimensions: None,
            min_dimensions: None,
            max_dimensions: None,
            vsync: default_vsync(),
            multisampling: default_multisampling(),
            visibility: default_visibility(),
        }
    }
}
fn default_title() -> String {
    "Amethyst game".to_string()
}
fn default_vsync() -> bool {
    true
}
fn default_multisampling() -> u16 {
    1
}
fn default_visibility() -> bool {
    true
}

impl DisplayConfig {
    /// Creates a `winit::WindowBuilder` using the values set in the DisplayConfig
    ///
    /// The EventsLoop is needed to configure a fullscreen window
    pub fn to_windowbuilder(self, el: winit::EventsLoop) -> WindowBuilder {
        use winit::WindowAttributes;
        let attrs = WindowAttributes {
            dimensions: self.dimensions.map(into_logical_size),
            max_dimensions: self.max_dimensions.map(into_logical_size),
            min_dimensions: self.min_dimensions.map(into_logical_size),
            title: self.title,
            visible: self.visibility,
            ..Default::default()
        };

        let mut builder = WindowBuilder::new();
        builder.window = attrs;

        if self.fullscreen {
            builder = builder.with_fullscreen(Some(el.get_primary_monitor()));
        }

        builder
    }
}

impl From<WindowBuilder> for DisplayConfig {
    fn from(wb: WindowBuilder) -> Self {
        DisplayConfig {
            title: wb.window.title,
            fullscreen: wb.window.fullscreen.is_some(),
            dimensions: wb.window.dimensions.map(into_dimensions),
            max_dimensions: wb.window.max_dimensions.map(into_dimensions),
            min_dimensions: wb.window.min_dimensions.map(into_dimensions),
            visibility: wb.window.visible,
            ..Default::default()
        }
    }
}

fn into_logical_size<D: Into<LogicalSize>>(dimensions: D) -> LogicalSize {
    dimensions.into()
}

fn into_dimensions<S: Into<(u32, u32)>>(size: S) -> (u32, u32) {
    size.into()
}
