//! Renderer configuration.

use glutin::WindowBuilder;

/// Structure for holding the renderer configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    /// Name of the application window.
    pub title: String,
    /// Enables or disables fullscreen mode.
    pub fullscreen: bool,
    /// Window dimensions, measured in pixels (px).
    pub dimensions: Option<(u32, u32)>,
    /// Maximum window dimensions, measured in pixels (px).
    pub max_dimensions: Option<(u32, u32)>,
    /// Minimum window dimensions, measured in pixels (px).
    pub min_dimensions: Option<(u32, u32)>,
    /// Enables or disables vertical synchronization.
    pub vsync: bool,
    /// Level of MSAA anti-aliasing.
    pub multisampling: u16,
    /// Sets the visibility of the window.
    pub visible: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: "Amethyst".to_string(),
            fullscreen: false,
            dimensions: None,
            max_dimensions: None,
            min_dimensions: None,
            vsync: true,
            multisampling: 0,
            visible: true,
        }
    }
}

impl From<Config> for WindowBuilder {
    fn from(cfg: Config) -> Self {
        use glutin::{self, WindowAttributes};

        let attrs = WindowAttributes {
            dimensions: cfg.dimensions,
            max_dimensions: cfg.max_dimensions,
            min_dimensions: cfg.min_dimensions,
            title: cfg.title,
            visible: cfg.visible,
            .. Default::default()
        };

        let mut builder = WindowBuilder::new();
        builder.window = attrs;

        if cfg.fullscreen {
            builder = builder.with_fullscreen(glutin::get_primary_monitor());
        }

        builder
    }
}

impl From<WindowBuilder> for Config {
    fn from(wb: WindowBuilder) -> Self {
        Config {
            title: wb.window.title,
            fullscreen: wb.window.monitor.is_some(),
            dimensions: wb.window.dimensions,
            max_dimensions: wb.window.max_dimensions,
            min_dimensions: wb.window.min_dimensions,
            visible: wb.window.visible,
            .. Default::default()
        }
    }
}
