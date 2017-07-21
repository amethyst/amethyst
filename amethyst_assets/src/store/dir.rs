use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use store::{Allocator, Store, StoreId};
use asset::MediaType;

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

    fn modified(
        &self,
        category: &str,
        id: &str,
        media_extensions: &[(&MediaType, &[&str])],
    ) -> Result<u64, IoError> {
        let mut path = self.loc.clone();

        path.push(category);
        path.push(id);

        for &(_, extensions) in media_extensions {
            for extension in extensions {
                match fs::metadata(path.with_extension(extension)) {
                    Ok(meta) => {
                        return Ok(
                            meta.modified()?
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        )
                    }
                    Err(ref err) if err.kind() == IoErrorKind::NotFound => {}
                    Err(err) => return Err(err),
                }
            }
        }

        Err(IoError::new(
            IoErrorKind::NotFound,
            format!(
                "no file {:?} with any of extensions {:?} found",
                path,
                media_extensions
            ),
        ))
    }

    fn store_id(&self) -> StoreId {
        self.id
    }

    fn load(
        &self,
        category: &str,
        name: &str,
        media_extensions: &'static [(&MediaType, &[&str])],
    ) -> Result<(&'static MediaType, Vec<u8>), IoError> {
        use std::io::Read;

        let mut path = self.loc.clone();

        path.push(category);
        path.push(name);

        for &(media_type, extensions) in media_extensions {
            for extension in extensions {
                match File::open(&path.with_extension(extension)) {
                    Ok(mut file) => {
                        let mut v = Vec::new();
                        file.read_to_end(&mut v)?;
                        return Ok((media_type, v));
                    }
                    Err(ref err) if err.kind() == IoErrorKind::NotFound => {}
                    Err(err) => return Err(err),
                }
            }
        }

        Err(IoError::new(
            IoErrorKind::NotFound,
            format!(
                "no file {:?} with any of extensions {:?} found",
                path,
                media_extensions
            ),
        ))
    }
}
