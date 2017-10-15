use std::fs::File;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use BoxedErr;
use source::Source;

/// Directory source.
///
/// Please note that there is a default directory source
/// inside the `Loader`, which is automatically used when you call
/// `load`. In case you want another, second, directory for assets,
/// you can instantiate one yourself, too. Please use `Loader::load_from` then.
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

    fn path(&self, s_path: &str) -> PathBuf {
        let mut path = self.loc.clone();
        path.push(s_path);

        path
    }
}

impl Source for Directory {
    fn modified(&self, path: &str) -> Result<u64, BoxedErr> {
        use std::fs::metadata;

        Ok(
            metadata(self.path(path))
                .map_err(BoxedErr::new)?
                .modified()
                .map_err(BoxedErr::new)?
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }

    fn load(&self, path: &str) -> Result<Vec<u8>, BoxedErr> {
        use std::io::Read;

        let mut v = Vec::new();
        let mut file = File::open(self.path(path)).map_err(BoxedErr::new)?;
        file.read_to_end(&mut v).map_err(BoxedErr::new)?;
        Ok(v)
    }
}
