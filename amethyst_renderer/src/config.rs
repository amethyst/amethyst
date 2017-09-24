//! Renderer configuration.

use winit::WindowBuilder;

/// Structure for holding the renderer configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
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
    /// FPS will be restricted to this value. Defaults to 144.
    #[serde(default = "default_max_fps")]
    pub max_fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            title: default_title(),
            fullscreen: false,
            dimensions: None,
            min_dimensions: None,
            max_dimensions: None,
            vsync: default_vsync(),
            multisampling: default_multisampling(),
            visibility: default_visibility(),
            max_fps: default_max_fps(),
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
fn default_max_fps() -> u32 {
    144
}

impl From<Config> for WindowBuilder {
    fn from(cfg: Config) -> Self {
        use winit::{self, WindowAttributes};

        let attrs = WindowAttributes {
            dimensions: cfg.dimensions,
            max_dimensions: cfg.max_dimensions,
            min_dimensions: cfg.min_dimensions,
            title: cfg.title,
            visible: cfg.visibility,
            ..Default::default()
        };

        let mut builder = WindowBuilder::new();
        builder.window = attrs;

        if cfg.fullscreen {
            builder = builder.with_fullscreen(winit::get_primary_monitor());
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
            visibility: wb.window.visible,
            ..Default::default()
        }
    }
}
