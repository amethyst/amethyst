use std::fs::File;
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use store::Store;

/// Directory store.
///
/// Please note that there is a default directory storage
/// inside the `Loader`, which is automatically used when you call
/// `load`. In case you want another, second, directory for assets,
/// you can instantiate one yourself, too. Please use `Loader::load_from`
/// then.
#[derive(Debug)]
pub struct Directory {
    loc: PathBuf,
}

impl Directory {
    /// Creates a new directory storage.
    pub fn new<P>(loc: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Directory { loc: loc.into() }
    }
}

impl Store for Directory {
    type Error = IoError;
    type Result = Result<Vec<u8>, IoError>;

    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, IoError> {
        use std::fs::metadata;

        let mut path = self.loc.clone();

        path.push(category);
        path.push(id);
        path.set_extension(ext);

        Ok(
            metadata(&path)?
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }

    fn load(&self, category: &str, name: &str, exts: &[&str]) -> Result<Vec<u8>, IoError> {
        use std::io::Read;

        let mut path = self.loc.clone();

        path.push(category);
        path.push(name);
        for ext in exts {
            path.set_extension(ext);

            let mut v = Vec::new();
            match File::open(&path) {
                Ok(mut file) => {
                    file.read_to_end(&mut v)?;
                    return Ok(v);
                }
                Err(io_error) => if io_error.kind() != ErrorKind::NotFound {
                    return Err(io_error);
                },
            }
        }
        Err(IoError::new(
            ErrorKind::NotFound,
            "Unable to find a file matching that path and any of the extensions for the format.",
        ))
    }
}
