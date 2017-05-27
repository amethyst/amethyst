//! Project management module
//!
//! Manages configuration files, paths, and such.

use std::io;
use std::path::{Path, PathBuf};

/// Configuration macros.
#[macro_use]
pub mod config;
/// Directory and file loading.
pub mod directory;

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

impl From<io::Error> for ProjectError {
    fn from(e: io::Error) -> ProjectError {
        ProjectError::File(e)
    }
}

impl ProjectError {
    /// Returns a human friendly error message for the `ProjectError`.
    pub fn description(&self) -> String {
        match self {
            &ProjectError::File(ref err) => err.to_string(),
            &ProjectError::Parser(ref msg) => msg.clone(),
        }
    }
}

/// Project structure, holds information related to configurations and meta information.
#[derive(Debug)]
pub struct Project {
    project: PathBuf,
    config: PathBuf,
}

impl Project {
    /// Takes in a path to signify the project directory.
    pub fn new<P: AsRef<Path>>(path: P) -> Project {
        let path = path.as_ref().to_path_buf();
        let mut configs = path.clone();
        configs.push("/resources/config/");
        
        Project {
            project: path,
            config: configs,
        }
    }
}
