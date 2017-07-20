use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use store::{Allocator, Store, StoreId};

/// Directory store.
///
/// Please note that there is a default directory storage
/// inside the `Loader`, which is automatically used when you call
/// `load`. In case you want another, second, directory for assets,
/// you can instantiate one yourself, too. Please use `Loader::load_from`
/// then.
#[derive(Debug)]
pub struct Directory {
    id: StoreId,
    loc: PathBuf,
}

impl Directory {
    /// Creates a new directory storage.
    pub fn new<P>(alloc: &Allocator, loc: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Directory {
            id: alloc.next_store_id(),
            loc: loc.into(),
        }
    }
}

impl Store for Directory {
    type Error = IoError;

    fn modified(&self, category: &str, id: &str, exts: &[&str]) -> Result<u64, IoError> {
        let mut path = self.loc.clone();

        path.push(category);
        path.push(id);

        for ext in exts {
            match fs::metadata(path.with_extension(ext)) {
                Ok(meta) => {
                    return Ok(
                        meta.modified()?
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    );
                }
                Err(_) => (),
            }
        }

        Err(IoError::new(
            IoErrorKind::NotFound,
            format!(
                "no file {:?} with any of extensions {:?} found",
                path,
                exts
            ),
        ))
    }

    fn store_id(&self) -> StoreId {
        self.id
    }

    fn load(&self, category: &str, name: &str, exts: &[&str]) -> Result<Vec<u8>, IoError> {
        use std::io::Read;

        let mut path = self.loc.clone();

        path.push(category);
        path.push(name);

        for ext in exts {
            match File::open(&path.with_extension(ext)) {
                Ok(mut file) => {
                    let mut v = Vec::new();
                    file.read_to_end(&mut v)?;
                    return Ok(v);
                }
                Err(_) => (),
            }
        }

        Err(IoError::new(
            IoErrorKind::NotFound,
            format!(
                "no file {:?} with any of extensions {:?} found",
                path,
                exts
            ),
        ))
    }
}
