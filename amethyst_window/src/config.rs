use std::path::PathBuf;

use log::error;
use serde::{Deserialize, Serialize};
use winit::window::{Icon, WindowAttributes, WindowBuilder};

use crate::{MonitorIdent, MonitorsAccess};

/// Configuration for a window display.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DisplayConfig {
    /// Name of the application window.
    #[serde(default = "default_title")]
    pub title: String,
    /// Enables fullscreen mode on specific monitor when set.
    /// Defaults to `None`, which means fullscreen is off.
    #[serde(default)]
    // pub fullscreen: Option<MonitorIdent>,
    pub fullscreen: Option<u32>,
    /// Current window dimensions, measured in pixels (px).
    #[serde(default)]
    pub dimensions: Option<(u32, u32)>,
    /// Minimum window dimensions, measured in pixels (px).
    #[serde(default)]
    pub min_dimensions: Option<(u32, u32)>,
    /// Maximum window dimensions, measured in pixels (px).
    #[serde(default)]
    pub max_dimensions: Option<(u32, u32)>,
    /// Whether the window should be immediately visible upon creation.
    #[serde(default = "default_visibility")]
    pub visibility: bool,
    /// A path to the icon used for the window.
    /// If `loaded_icon` is present, this will be ignored.
    #[serde(default)]
    pub icon: Option<PathBuf>,
    /// Whether the window should always be on top of other windows.
    #[serde(default)]
    pub always_on_top: bool,
    /// Whether the window should have borders and bars.
    #[serde(default = "default_decorations")]
    pub decorations: bool,
    /// Whether the window should be maximized upon creation.
    #[serde(default)]
    pub maximized: bool,
    /// Enable multitouch on iOS.
    #[serde(default)]
    pub multitouch: bool,
    /// Whether the window is resizable or not.
    #[serde(default = "default_resizable")]
    pub resizable: bool,
    /// Whether the the window should be transparent. If this is true, writing
    /// colors with alpha values different than 1.0 will produce a transparent
    /// window.
    #[serde(default)]
    pub transparent: bool,

    /// A programmatically loaded window icon; not present in serialization.
    /// Takes precedence over `icon`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use amethyst_window::{DisplayConfig, Icon};
    ///
    /// // First, create your `DisplayConfig` as usual
    /// let mut config = DisplayConfig::default(); // or load from file
    ///
    /// // Create the icon data
    /// let mut icon = Vec::new();
    /// for _ in 0..(128 * 128) {
    ///     icon.extend(vec![255, 0, 0, 255]);
    /// }
    ///
    /// // Set the `loaded_icon` field of the config
    /// // It will now be used as the window icon
    /// config.loaded_icon = Some(Icon::from_rgba(icon, 128, 128).unwrap());
    ///
    /// // Now, feed this into the `GameDataBuilder` using
    /// // `.with_bundle(WindowBundle::from_config(config))`
    /// ```
    #[serde(skip)]
    pub loaded_icon: Option<Icon>,
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
            icon: None,
            always_on_top: false,
            decorations: default_decorations(),
            maximized: false,
            multitouch: false,
            resizable: default_resizable(),
            transparent: false,
            loaded_icon: None,
        }
    }
}

fn default_title() -> String {
    "Amethyst game".to_string()
}

fn default_decorations() -> bool {
    true
}

fn default_visibility() -> bool {
    true
}

fn default_resizable() -> bool {
    true
}

impl DisplayConfig {
    /// Creates a `winit::WindowBuilder` using the values set in the `DisplayConfig`.
    ///
    /// The `MonitorsAccess` is needed to configure a fullscreen window.
    pub fn into_window_builder(self, monitors: &impl MonitorsAccess) -> WindowBuilder {
        let attrs = WindowAttributes {
            inner_size: self.dimensions.map(Into::into),
            max_inner_size: self.max_dimensions.map(Into::into),
            min_inner_size: self.min_dimensions.map(Into::into),
            resizable: self.resizable,
            fullscreen: None,
            title: self.title,
            maximized: self.maximized,
            visible: self.visibility,
            transparent: self.transparent,
            decorations: self.decorations,
            always_on_top: self.always_on_top,
            window_icon: None,
        };

        let mut builder = WindowBuilder::new();
        builder.window = attrs;

        if self.loaded_icon.is_some() {
            builder = builder.with_window_icon(self.loaded_icon);
        } else if let Some(icon) = self.icon {
            // let icon = match Icon::from_path(&icon) {
            //     Ok(x) => Some(x),
            //     Err(e) => {
            //         error!(
            //             "Failed to load window icon from `{}`: {}",
            //             icon.display(),
            //             e
            //         );

            //         None
            //     }
            // };

            // builder = builder.with_window_icon(icon);
        }

        builder
    }
}
