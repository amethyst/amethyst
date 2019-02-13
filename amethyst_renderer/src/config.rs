//! Renderer configuration.

use serde::{Deserialize, Serialize};
use winit::{self, Icon, MonitorId, WindowAttributes, WindowBuilder};

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
///     vsync: true,
///     always_on_top: false,
///     decorations: true,
///     maximized: false,
///     multitouch: true,
///     resizable: true,
///     transparent: false,
/// )
/// ```
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct DisplayConfig {
    /// Name of the application window.
    pub title: String,

    /// Enables or disables fullscreen mode.
    pub fullscreen: bool,

    /// Current window dimensions, measured in pixels (px).
    pub dimensions: Option<(u32, u32)>,

    /// Minimum window dimensions, measured in pixels (px).
    pub min_dimensions: Option<(u32, u32)>,

    /// Maximum window dimensions, measured in pixels (px).
    pub max_dimensions: Option<(u32, u32)>,

    /// Path to window icon.
    pub icon: Option<String>,

    /// Window icon. This must be set before render initialization and takes precedence over `icon`.
    #[serde(skip)]
    pub loaded_icon: Option<Icon>,

    /// Enables or disables vertical synchronization.
    pub vsync: bool,

    /// Level of MSAA anti-aliasing.
    pub multisampling: u16,

    /// Sets the visibility of the window.
    pub visibility: bool,

    /// Whether the window should always be on top of other windows.
    pub always_on_top: bool,

    /// Whether the window should have borders and bars.
    pub decorations: bool,

    /// Whether the window should be maximized upon creation.
    pub maximized: bool,

    /// Enable multitouch on iOS.
    pub multitouch: bool,

    /// Whether the window is resizable or not.
    pub resizable: bool,

    /// Whether the the window should be transparent. If this is true, writing
    /// colors with alpha values different than 1.0 will produce a transparent
    /// window.
    pub transparent: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            always_on_top: false,
            decorations: true,
            dimensions: Some((640, 480)),
            fullscreen: false,
            icon: None,
            loaded_icon: None,
            max_dimensions: None,
            maximized: false,
            min_dimensions: None,
            multisampling: 1,
            multitouch: true,
            resizable: true,
            title: "Amethyst game".to_string(),
            transparent: false,
            visibility: true,
            vsync: true,
        }
    }
}

impl DisplayConfig {
    /// Creates a `WindowBuilder` using the values set in the DisplayConfig
    ///
    /// The `MonitorId` is needed to configure a fullscreen window
    pub fn to_windowbuilder(self, monitor_id: MonitorId) -> WindowBuilder {
        let attrs = WindowAttributes {
            always_on_top: self.always_on_top,
            decorations: self.decorations,
            dimensions: self.dimensions.map(|x| x.into()),
            fullscreen: None,
            max_dimensions: self.max_dimensions.map(|x| x.into()),
            maximized: self.maximized,
            min_dimensions: self.min_dimensions.map(|x| x.into()),
            multitouch: self.multitouch,
            resizable: self.resizable,
            title: self.title,
            transparent: self.transparent,
            visible: self.visibility,
            window_icon: None,
        };

        let mut builder = WindowBuilder::new();
        builder.window = attrs;

        if self.fullscreen {
            builder = builder.with_fullscreen(Some(monitor_id));
        }

        if self.loaded_icon.is_some() {
            builder = builder.with_window_icon(self.loaded_icon);
        } else if let Some(icon) = self.icon {
            builder = builder.with_window_icon(Icon::from_path(icon).ok());
        }

        builder
    }
}

impl From<WindowBuilder> for DisplayConfig {
    fn from(wb: WindowBuilder) -> Self {
        DisplayConfig {
            always_on_top: wb.window.always_on_top,
            decorations: wb.window.decorations,
            dimensions: wb.window.dimensions.map(|x| x.into()),
            fullscreen: wb.window.fullscreen.is_some(),
            max_dimensions: wb.window.max_dimensions.map(|x| x.into()),
            maximized: wb.window.maximized,
            min_dimensions: wb.window.min_dimensions.map(|x| x.into()),
            multitouch: wb.window.multitouch,
            resizable: wb.window.resizable,
            title: wb.window.title,
            transparent: wb.window.transparent,
            visibility: wb.window.visible,
            ..Default::default()
        }
    }
}
