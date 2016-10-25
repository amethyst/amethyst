
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use ::project::ProjectError;

/// Tracking root folder for files as well as a list of files in destination.
#[derive(Debug)]
pub struct Directory {
    /// Root directory of the file system
    pub root: PathBuf,

    /// Whether recursively check subfolders
    ///
    /// non-recursive:
    /// ```
    /// resources/
    ///   config/
    ///     config.yml
    ///     display.yml
    ///     logging.yml
    /// ```
    ///
    /// recursive:
    /// ```
    /// resources/
    ///   config/
    ///     config.yml
    ///     display/
    ///       config.yml # same as "resources/config/display.yml"
    ///     logging.yml
    /// ```
    pub recursive: bool,

    /// Extensions of files it will look for.
    pub extensions: Vec<String>,

    /// list of files found
    pub list: Vec<PathBuf>,
}

impl Directory {
    pub fn new<P: AsRef<Path>>(path: P,
                               recursive: bool,
                               extensions: Vec<String>)
                               -> Result<Directory, ProjectError> {
        let mut directory = Directory {
            root: path.as_ref().into(),
            recursive: recursive,
            extensions: extensions,
            list: Vec::new(),
        };

        try!(directory.refresh());
        Ok(directory)
    }

    /// Re-check directory for yaml files and refresh contents
    pub fn refresh(&mut self) -> Result<(), ProjectError> {
        let list =
            try!(Directory::files_in_dir(self.root.clone(), self.recursive, &self.extensions));
        for entry in list {
            self.list.push(entry.clone());
        }

        Ok(())
    }

    /// Return contents of a file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<String, ProjectError> {
        let mut file = try!(fs::File::open(&path));
        let mut buffer = String::new();
        try!(file.read_to_string(&mut buffer));
        Ok(buffer)
    }

    /// Gets a list of files in a directory
    pub fn files_in_dir(path: PathBuf,
                        recursive: bool,
                        extensions: &Vec<String>)
                        -> Result<Vec<PathBuf>, ProjectError> {
        let mut list = Vec::new();
        for entry in try!(fs::read_dir(&path)) {
            let dir = try!(entry);
            let path = dir.path();

            if let Ok(file_type) = dir.file_type() {
                if file_type.is_file() {
                    if let Some(extension) = path.extension() {
                        if extensions.contains(&extension.to_string_lossy().to_string()) {
                            list.push(path.clone());
                        }
                    }
                } else if file_type.is_dir() && recursive {
                    list.extend(try!(Directory::files_in_dir(path, recursive, extensions)));
                }
            }
        }

        Ok(list)
    }
}
