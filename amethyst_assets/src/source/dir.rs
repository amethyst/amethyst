use std::{
    fs::File,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use amethyst_error::{format_err, Error, ResultExt};

use crate::{error, source::Source};

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
    fn modified(&self, path: &str) -> Result<u64, Error> {
        use std::fs::metadata;

        let path = self.path(path);

        metadata(&path)
            .with_context(|_| format_err!("Failed to fetch metadata for {:?}", path))?
            .modified()
            .with_context(|_| format_err!("Could not get modification time"))?
            .duration_since(UNIX_EPOCH)
            .with_context(|_| {
                format_err!("Anomalies with the system clock caused `duration_since` to fail")
            })
            .map(|d| d.as_secs())
    }

    fn load(&self, path: &str) -> Result<Vec<u8>, Error> {
        use std::io::Read;

        use encoding_rs_io::DecodeReaderBytes;

        let path = self.path(path);

        let mut v = Vec::new();
        let file = File::open(&path)
            .with_context(|_| format_err!("Failed to open file {:?}", path))
            .with_context(|_| error::Error::Source)?;

        // If UTF-8-BOM or UTF-16-BOM then convert to regular UTF-8. Else bytes are passed through
        let mut decoder = DecodeReaderBytes::new(file);

        decoder
            .read_to_end(&mut v)
            .with_context(|_| format_err!("Failed to read file {:?}", path))
            .with_context(|_| error::Error::Source)?;

        Ok(v)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::Directory;
    use crate::source::Source;

    #[test]
    fn loads_asset_from_assets_directory() {
        let test_assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets");
        let directory = Directory::new(test_assets_dir);

        assert_eq!(
            b"data".to_vec(),
            directory
                .load("subdir/asset")
                .expect("Failed to load tests/assets/subdir/asset")
        );
    }

    #[test]
    fn load_assets_with_bom_encodings() {
        let test_assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets");
        let directory = Directory::new(test_assets_dir);

        assert_eq!(
            b"amethyst".to_vec(),
            directory
                .load("encodings/UTF8-BOM")
                .expect("Failed to parse UTF8 file with BOM")
        );
        assert_eq!(
            b"amethyst".to_vec(),
            directory
                .load("encodings/UTF16-LE-BOM")
                .expect("Failed to parse UTF16-LE file with BOM")
        );
        assert_eq!(
            b"amethyst".to_vec(),
            directory
                .load("encodings/UTF16-BE-BOM")
                .expect("Failed to parse UTF16-BE file with BOM")
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
            b"data".to_vec(),
            // Use forward slash to declare path
            directory
                .load("subdir/asset")
                .expect("Failed to load tests/assets/subdir/asset")
        );
    }
}
