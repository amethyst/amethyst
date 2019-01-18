use std::{
    fs::File,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{source::Source, ErrorKind, Result, ResultExt};

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
        path.extend(Path::new(s_path).iter());

        path
    }
}

impl Source for Directory {
    fn modified(&self, path: &str) -> Result<u64> {
        #[cfg(feature = "profiler")]
        profile_scope!("dir_modified_asset");
        use std::fs::metadata;

        let path = self.path(path);

        metadata(&path)
            .chain_err(|| format!("Failed to fetch metadata for {:?}", path))?
            .modified()
            .chain_err(|| "Could not get modification time")?
            .duration_since(UNIX_EPOCH)
            .chain_err(|| "Anomalies with the system clock caused `duration_since` to fail")
            .map(|d| d.as_secs())
    }

    fn load(&self, path: &str) -> Result<Vec<u8>> {
        #[cfg(feature = "profiler")]
        profile_scope!("dir_load_asset");
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

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::source::Source;

    use super::Directory;

    #[test]
    fn loads_asset_from_assets_directory() {
        let test_assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets");
        let directory = Directory::new(test_assets_dir);

        assert_eq!(
            "data".as_bytes().to_vec(),
            directory
                .load("subdir/asset")
                .expect("Failed to load tests/assets/subdir/asset")
        );
    }

    #[cfg(windows)]
    #[test]
    fn tolerates_backslashed_location_with_forward_slashed_asset_paths() {
        // Canonicalized path on Windows uses backslashes
        let test_assets_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/assets")
            .canonicalize()
            .expect("Failed to canonicalize tests/assets directory");
        let directory = Directory::new(test_assets_dir);

        assert_eq!(
            "data".as_bytes().to_vec(),
            // Use forward slash to declare path
            directory
                .load("subdir/asset")
                .expect("Failed to load tests/assets/subdir/asset")
        );
    }
}
