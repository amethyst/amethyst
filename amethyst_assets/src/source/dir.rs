use std::fs::File;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use {ErrorKind, Result, ResultExt};
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
    fn modified(&self, path: &str) -> Result<u64> {
        use std::fs::metadata;

        let path = self.path(path);

        Ok(
            metadata(&path)
                .chain_err(|| format!("Failed to fetch metadata for {:?}", path))?
                .modified()
                .chain_err(|| "Could not get modification time")?
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }

    fn load(&self, path: &str) -> Result<Vec<u8>> {
        use std::io::Read;

        let path = self.path(path);

        let mut v = Vec::new();
        let mut file = File::open(&path)
            .chain_err(|| format!("Failed to open file {:?}", path))
            .chain_err(|| ErrorKind::Source)?;
        file.read_to_end(&mut v)
            .chain_err(|| format!("Failed to read file {:?}", path))
            .chain_err(|| ErrorKind::Source)?;

        Ok(v)
    }
}
