//! Project management module
//!
//! Manages configuration files, paths, and such.

use std::io;
use std::fmt;
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
}

impl ProjectError {
    /// Displays the type of error and relevant information.
    pub fn to_string(&self) -> &str {
        match self {
            &ProjectError::File(ref err) => err.description(),
            &ProjectError::Parser(ref msg) => msg,
        }
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string()) 
    }
}

impl From<io::Error> for ProjectError {
    fn from(e: io::Error) -> ProjectError {
        ProjectError::File(e)
    }
}

impl Error for ProjectError {
    /// Returns a human friendly error message for the `ProjectError`.
    fn description(&self) -> &str {
        self.to_string()
    }
}

