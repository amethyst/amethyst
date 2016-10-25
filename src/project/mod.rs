
use std::io;

#[macro_use]
pub mod config;
pub mod directory;

#[derive(Debug)]
pub enum ProjectError {
    File(io::Error),
    Parser(String),
}

impl From<io::Error> for ProjectError {
    fn from(e: io::Error) -> ProjectError {
        ProjectError::File(e)
    }
}

