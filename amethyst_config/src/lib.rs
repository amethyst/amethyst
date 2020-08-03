//! Loads RON files into a structure for easy / statically typed usage.

#![crate_name = "amethyst_config"]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
};

#[cfg(feature = "binary")]
use bincode::Error as BincodeError;
use ron::{self, de::Error as DeError, ser::Error as SerError};
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use serde_json::error::Error as SerJsonError;

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
    /// Forward to serde json's errors
    #[cfg(feature = "json")]
    SerdeJsonError(SerJsonError),
    /// Forward to bincode's errors
    #[cfg(feature = "binary")]
    BincodeError(BincodeError),
}

/// Config file format for serde
#[derive(Debug)]
pub enum ConfigFormat {
    /// Rusty Object Notation files (.ron), default
    Ron,
    /// JavaScript Object Notation files (.json), requires enabling `json` feature
    #[cfg(feature = "json")]
    Json,
    /// Binary files (.bin), encoded with bincode, requires enabling `binary` feature
    #[cfg(feature = "binary")]
    Binary,
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
            #[cfg(feature = "json")]
            ConfigError::SerdeJsonError(ref msg) => write!(f, "{}", msg),
            #[cfg(feature = "binary")]
            ConfigError::BincodeError(ref msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "json")]
impl From<SerJsonError> for ConfigError {
    fn from(e: SerJsonError) -> Self {
        ConfigError::SerdeJsonError(e)
    }
}

#[cfg(feature = "binary")]
impl From<BincodeError> for ConfigError {
    fn from(e: BincodeError) -> Self {
        ConfigError::BincodeError(e)
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
            #[cfg(feature = "json")]
            ConfigError::SerdeJsonError(_) => "Serialization or deserialization error (serde_json)",
            #[cfg(feature = "binary")]
            ConfigError::BincodeError(_) => "Serialization or deserialization error (bincode)",
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
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;

    /// Loads a configuration structure from a file.
    #[deprecated(note = "use `load` instead")]
    fn load_no_fallback<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        Self::load(path)
    }

    /// Loads configuration structure from raw bytes.
    fn load_bytes(format: ConfigFormat, bytes: &[u8]) -> Result<Self, ConfigError>;

    /// Writes a configuration structure to a file in RON format.
    #[deprecated(note = "use `write_format` instead")]
    fn write<P: AsRef<Path>>(&self, format: ConfigFormat, path: P) -> Result<(), ConfigError> {
        self.write_format(ConfigFormat::Ron, path)
    }

    /// Writes a configuration structure to a file.
    fn write_format<P: AsRef<Path>>(
        &self,
        format: ConfigFormat,
        path: P,
    ) -> Result<(), ConfigError>;
}

impl<T> Config for T
where
    T: for<'a> Deserialize<'a> + Serialize,
{
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        use std::{fs::File, io::Read};

        let path = path.as_ref();

        let content = {
            let mut file = File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            buffer
        };

        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            match extension {
                "ron" => Self::load_bytes(ConfigFormat::Ron, &content),
                #[cfg(feature = "json")]
                "json" => Self::load_bytes(ConfigFormat::Json, &content),
                #[cfg(feature = "binary")]
                "bin" => Self::load_bytes(ConfigFormat::Binary, &content),
                _ => Err(ConfigError::Extension(path.to_path_buf())),
            }
        } else {
            Err(ConfigError::Extension(path.to_path_buf()))
        }
    }

    fn load_bytes(format: ConfigFormat, bytes: &[u8]) -> Result<Self, ConfigError> {
        match format {
            ConfigFormat::Ron => {
                let mut de = ron::de::Deserializer::from_bytes(bytes)?;
                let des = T::deserialize(&mut de)?;
                de.end()?;
                Ok(des)
            }
            #[cfg(feature = "json")]
            ConfigFormat::Json => {
                let mut de = serde_json::de::Deserializer::from_slice(bytes);
                let des = T::deserialize(&mut de)?;
                de.end()?;
                Ok(des)
            }
            #[cfg(feature = "binary")]
            ConfigFormat::Binary => {
                use bincode::config::Options;
                let des: T = bincode::deserialize(bytes)?;
                Ok(des)
            }
        }
    }

    fn write_format<P: AsRef<Path>>(
        &self,
        format: ConfigFormat,
        path: P,
    ) -> Result<(), ConfigError> {
        use std::{fs::File, io::Write};

        match format {
            ConfigFormat::Ron => {
                let str = ron::ser::to_string_pretty(self, Default::default())?;
                File::create(path)?.write_all(str.as_bytes())?;
            }
            #[cfg(feature = "json")]
            ConfigFormat::Json => {
                let str = serde_json::ser::to_string_pretty(self)?;
                File::create(path)?.write_all(str.as_bytes())?;
            }
            #[cfg(feature = "binary")]
            ConfigFormat::Binary => File::create(path)?.write_all(&bincode::serialize(self)?)?,
        };

        Ok(())
    }
}
