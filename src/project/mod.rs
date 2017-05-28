//! Project management module
//!
//! Manages configuration files, paths, and such.

use std::io;
use std::fmt;
use std::path::{PathBuf};
use std::error::Error;

/// Configuration macros.
#[macro_use]
pub mod config;

pub use self::config::Config;

/// Error related to anything that manages/creates configurations as well as
/// "workspace"-related things.
#[derive(Debug)]
pub enum ProjectError {
    /// Forward to the `std::io::Error` error.
    File(io::Error),
    /// Errors related to serde's parsing of configuration files.
    Parser(String),
    /// Related to the path of the file.
    Extension(PathBuf),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ProjectError::File(ref err) => write!(f, "{}", err),
            ProjectError::Parser(ref msg) => write!(f, "{}", msg),
            ProjectError::Extension(ref path) => {
                let found = match path.extension() {
                    Some(extension) => format!("{:?}", extension),
                    None => format!("a directory."),
                };

                write!(
                    f,
                    "{}: Invalid path extension, expected \"yml\", \"yaml\", or \"toml\". Got {}. ", 
                    path.display().to_string(),
                    found,
                )
            },
        }
    }
}

impl From<io::Error> for ProjectError {
    fn from(e: io::Error) -> ProjectError {
        ProjectError::File(e)
    }
}

impl Error for ProjectError {
    fn description(&self) -> &str {
        match *self {
            ProjectError::File(_) => "Project file error",
            ProjectError::Parser(_) => "Project parser error",
            ProjectError::Extension(_) => "Invalid extension or directory for a file",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ProjectError::File(ref err) => Some(err),
            _ => None,
        }
    }
}

