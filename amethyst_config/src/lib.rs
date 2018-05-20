//! Loads RON files into a structure for easy / statically typed usage.
//!

#![crate_name = "amethyst_config"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://tinyurl.com/hgsb45k")]

extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
extern crate ron;
extern crate serde;

#[cfg(feature = "profiler")]
extern crate thread_profiler;

use std::path::Path;

pub use error::{Error, ErrorKind, Result, WrongExtension};
use failure::ResultExt;
use serde::{Deserialize, Serialize};

mod error;

/// Trait implemented by the `config!` macro.
pub trait Config
where
    Self: Sized,
{
    /// Loads a configuration structure from a file.
    /// Defaults if the file fails in any way.
    fn load<P: AsRef<Path>>(path: P) -> Self;

    /// Loads a configuration structure from a file.
    fn load_no_fallback<P: AsRef<Path>>(path: P) -> Result<Self>;

    /// Writes a configuration structure to a file.
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

impl<T> Config for T
where
    T: for<'a> Deserialize<'a> + Serialize + Default,
{
    fn load<P: AsRef<Path>>(path: P) -> Self {
        Self::load_no_fallback(path.as_ref()).unwrap_or_else(|e| {
            warn!("Failed to load config: {}", e);
            Self::default()
        })
    }

    fn load_no_fallback<P: AsRef<Path>>(path: P) -> Result<Self> {
        use ron::de::Deserializer;
        use std::fs::File;
        use std::io::Read;

        let path = path.as_ref();

        let content = {
            let mut file = File::open(path).context(ErrorKind::File)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).context(ErrorKind::File)?;

            buffer
        };

        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            let mut d = Deserializer::from_bytes(&content);
            let val = T::deserialize(&mut d).context(ErrorKind::Parser)?;
            d.end().context(ErrorKind::Parser)?;

            Ok(val)
        } else {
            Err(Error::wrong_extension(path.to_path_buf()))
        }
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use ron::ser::to_string_pretty;
        use std::fs::File;
        use std::io::Write;

        let s = to_string_pretty(self, Default::default()).context(ErrorKind::Serializer)?;
        File::create(path)
            .context(ErrorKind::Serializer)?
            .write(s.as_bytes())
            .context(ErrorKind::Serializer)?;

        Ok(())
    }
}
