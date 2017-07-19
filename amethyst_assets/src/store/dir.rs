use std::fs::File;
use std::io::Error as IoError;
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
        where P: Into<PathBuf>
    {
        Directory {
            id: alloc.next_store_id(),
            loc: loc.into(),
        }
    }
}

impl Store for Directory {
    type Error = IoError;

    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, IoError> {
        use std::fs::metadata;

        let mut path = self.loc.clone();

        path.push(category);
        path.push(id);
        path.set_extension(ext);

        Ok(metadata(&path)?.modified()?.duration_since(UNIX_EPOCH).unwrap().as_secs())
    }

    fn store_id(&self) -> StoreId {
        self.id
    }

    fn load(&self, category: &str, name: &str, ext: &str) -> Result<Vec<u8>, IoError> {
        use std::io::Read;

        let mut path = self.loc.clone();

        path.push(category);
        path.push(name);
        path.set_extension(ext);

        let mut v = Vec::new();
        File::open(&path)?.read_to_end(&mut v)?;

        Ok(v)
    }
}
