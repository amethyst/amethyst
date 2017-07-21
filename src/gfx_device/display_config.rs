//! Graphical display configuration.

config!(
    /// Graphical display configuration.
    ///
    /// These are fed in when calling `video_init()`.
    pub struct DisplayConfig {
        /// Name of the application window.
        pub title: String = "Amethyst game".to_string(),
        /// Enables or disables fullscreen mode.
        pub fullscreen: bool = false,
        /// Current window dimensions, measured in pixels (px).
        pub dimensions: Option<(u32, u32)> = None,
        /// Minimum window dimensions, measured in pixels (px).
        pub min_dimensions: Option<(u32, u32)> = None,
        /// Maximum window dimensions, measured in pixels (px).
        pub max_dimensions: Option<(u32, u32)> = None,
        /// Enables or disables vertical synchronization.
        pub vsync: bool = true,
        /// Level of MSAA anti-aliasing.
        pub multisampling: u16 = 1,
        /// Sets the visibility of the window.
        pub visibility: bool = true,
    }
);
