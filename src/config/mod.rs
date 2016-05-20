
use std::path::Path;
pub use yaml_rust::Yaml;

#[macro_use]
mod definitions;
mod yaml;

pub use config::yaml::{Element, to_string};
pub use config::definitions::{ConfigMeta, ConfigError};

// Defines types along with defaulting values
config!(DisplayConfig {
    brightness: f64 = 1.0,
    fullscreen: bool = false,
    size: [u16; 2] = [1024, 768],
});

config!(LoggingConfig {
    file_path: String = "new_project.log".to_string(),
    output_level: String = "warn".to_string(),
    logging_level: String = "debug".to_string(),
});

config!(InnerInnerConfig {
    field: u64 = 58123,
});

config!(InnerConfig {
    inner_inner: InnerInnerConfig = InnerInnerConfig::default(),
});

config!(Config {
    title: String = "Amethyst game".to_string(),
    display: DisplayConfig = DisplayConfig::default(),
    logging: LoggingConfig = LoggingConfig::default(),
    inner: InnerConfig = InnerConfig::default(),
    inner_inner: InnerInnerConfig = InnerInnerConfig::default(),
});