//! Graphical display configuration.

/// Graphical display configuration.
///
/// These are fed in when calling `video_init()`.
#[derive(Debug, Deserialize, Serialize)]
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
fn default_title() -> String { "Amethyst game".to_string() }
fn default_vsync() -> bool { true }
fn default_multisampling() -> u16 { 1 }
fn default_visibility() -> bool { true }

