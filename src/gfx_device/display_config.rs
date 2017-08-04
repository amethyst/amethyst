//! Graphical display configuration.

/// Graphical display configuration.
///
/// These are fed in when calling `video_init()`.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DisplayConfig {
    /// Name of the application window.
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
    #[serde(default)]
    pub vsync: bool,
    /// Level of MSAA anti-aliasing.
    #[serde(default)]
    pub multisampling: u16,
    /// Sets the visibility of the window.
    #[serde(default = "true")]
    pub visibility: bool,
}
