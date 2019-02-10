//! Loads RON files into a structure for easy / statically typed usage.

#![crate_name = "amethyst_config"]
#![doc(html_logo_url = "https://www.amethyst.rs/assets/amethyst.svg")]
#![warn(missing_docs, rust_2018_idioms, rust_2018_compatibility)]

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
};

use log::error;
use ron::{self, de::Error as DeError, ser::Error as SerError};
use serde::{Deserialize, Serialize};

/// Error related to anything that manages/creates configurations as well as
/// "workspace"-related things.
#[derive(Debug)]
pub enum ConfigError {
    /// Forward to the `std::io::Error` error.
    File(io::Error),
    /// Errors related to serde's parsing of configuration files.
    Parser(DeError),
    /// Occurs if a value is ill-formed during serialization (like a poisoned mutex).
    Serializer(SerError),
    /// Related to the path of the file.
    Extension(PathBuf),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ConfigError::File(ref err) => write!(f, "{}", err),
            ConfigError::Parser(ref msg) => write!(f, "{}", msg),
            ConfigError::Serializer(ref msg) => write!(f, "{}", msg),
            ConfigError::Extension(ref path) => {
                let found = match path.extension() {
                    Some(extension) => format!("{:?}", extension),
                    None => "a directory.".to_string(),
                };

                write!(
                    f,
                    "{}: Invalid path extension, expected \"ron\", got {}.",
                    path.display().to_string(),
                    found,
                )
            }
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        ConfigError::File(e)
    }
}

impl From<DeError> for ConfigError {
    fn from(e: DeError) -> Self {
        ConfigError::Parser(e)
    }
}

impl From<SerError> for ConfigError {
    fn from(e: SerError) -> Self {
        ConfigError::Serializer(e)
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::File(_) => "Project file error",
            ConfigError::Parser(_) => "Project parser error",
            ConfigError::Serializer(_) => "Project serializer error",
            ConfigError::Extension(_) => "Invalid extension or directory for a file",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            ConfigError::File(ref err) => Some(err),
            _ => None,
        }
    }
}

/// Trait implemented by the `config!` macro.
pub trait Config
where
    Self: Sized,
{
    /// Loads a configuration structure from a file.
    /// Defaults if the file fails in any way.
    fn load<P: AsRef<Path>>(path: P) -> Self;

    /// Loads a configuration structure from a file.
    fn load_no_fallback<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;

    /// Loads configuration structure from raw bytes.
    fn load_bytes(bytes: &[u8]) -> Result<Self, ConfigError>;

    /// Writes a configuration structure to a file.
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError>;
}

impl<T> Config for T
where
    T: for<'a> Deserialize<'a> + Serialize + Default,
{
    fn load<P: AsRef<Path>>(path: P) -> Self {
        Self::load_no_fallback(path.as_ref()).unwrap_or_else(|e| {
            if let Some(path) = path.as_ref().to_str() {
                error!("Failed to load config file '{}': {}", path, e);
            } else {
                error!("Failed to load config: {}", e);
            }

            Self::default()
        })
    }

    fn load_no_fallback<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        use std::{fs::File, io::Read};

        let path = path.as_ref();

        let content = {
            let mut file = File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            buffer
        };

        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            Self::load_bytes(&content)
        } else {
            Err(ConfigError::Extension(path.to_path_buf()))
        }
    }

    fn load_bytes(bytes: &[u8]) -> Result<Self, ConfigError> {
        let mut de = ron::de::Deserializer::from_bytes(bytes)?;
        let val = T::deserialize(&mut de)?;
        de.end()?;

        Ok(val)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        use ron::ser::to_string_pretty;
        use std::{fs::File, io::Write};

        let s = to_string_pretty(self, Default::default())?;
        File::create(path)?.write_all(s.as_bytes())?;

        Ok(())
    }
}
