
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

impl ProjectError {
    pub fn description(&self) -> String {
        match self {
            &ProjectError::File(ref err) => err.to_string(),
            &ProjectError::Parser(ref msg) => msg.clone(),
        }
    }
}

