
use std::fs::File;
use std::io::{Read, Error};
use std::path::{PathBuf, Path};
use std::default::Default;
use std::fmt;

use yaml_rust::{YamlLoader, ScanError};
pub use yaml_rust::Yaml;

#[macro_use]
mod definitions;
mod yaml;

pub use config::yaml::{FromYaml};
pub use config::definitions::{FromFile, ConfigMeta, ConfigError};

use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};

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

config!(Config {
    title: String = "Amethyst game".to_string(),
    display: DisplayConfig = DisplayConfig::default(),
    logging: LoggingConfig = LoggingConfig::default(),
});