//! Provided common things for asset management

use asset_manager::{Asset, AssetFormat, AssetStore, AssetStoreError};

// -----------------------------------------------------------------------------------------
// Asset stores
// -----------------------------------------------------------------------------------------

use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{BufReader, Error as IoError, ErrorKind, Read, Seek};

use zip::ZipArchive;

#[cfg(not(android))]
mod desktop {
    use super::*;

    use asset_manager::{AssetFormat, AssetStoreError};

    pub fn read_asset<T: Asset, F: AssetFormat>(name: &str,
                                                format: &F)
                                                -> Result<Box<[u8]>, AssetStoreError> {
        read_asset_from_path(Path::new("assets").join(T::category()).join(name), format)
    }

    pub fn read_asset_from_path<P, F>(path: P, format: &F) -> Result<Box<[u8]>, AssetStoreError>
        where P: AsRef<Path>,
              F: AssetFormat
    {
        let mut last_error = None;

        let mut path = PathBuf::from(path.as_ref());

        for extension in format.file_extensions() {
            path.set_extension(extension);

            println!("Trying path {:?}", &path);

            match read_file_complete(&path) {
                Ok(x) => return Ok(x),
                Err(AssetStoreError::NoSuchAsset) => continue,
                Err(AssetStoreError::PermissionDenied) => {
                    last_error = Some(AssetStoreError::PermissionDenied);
                    continue;
                }
                Err(x) => return Err(x),
            }
        }

        return Err(last_error.unwrap_or(AssetStoreError::NoSuchAsset));
    }
}

#[cfg(android)]
mod android {
    use super::*;

    use android_glue::AssetError;
    use asset_manager::{AssetFormat, AssetStoreError};

    pub fn read_asset<T: Asset, F: AssetFormat>(name: &str,
                                                format: &F)
                                                -> Result<Box<[u8]>, AssetStoreError> {
        for extension in format.file_extensions() {
            let file_name = into_file_name(name, extension);
            let path = PathBuf::from(T::category()).join(file_name);

            match android_glue::load_asset(path.to_str().unwrap()) {
                Ok(x) => Ok(x.into_boxed_slice()),
                Err(AssetError::AssetMissing) => continue,
                Err(AssetError::EmptyBuffer) => {
                    Err(AssetStoreError::Other("EmptyBuffer".to_string()))
                }
            }

            return Err(AssetStoreError::NoSuchAsset);
        }
    }

    pub fn read_bytes(path: &str) -> Result<Box<[u8]>, IoError> {
        match android_glue::load_asset(path) {
            Ok(x) => Ok(x.into_boxed_slice()),
            Err(AssetError::AssetMissing) => Err(ErrorKind::NotFound.into()),
            Err(AssetError::EmptyBuffer) => Err(ErrorKind::InvalidData.into()),
        }
    }

    fn into_file_name<F: AssetFormat>(name: &str, extension: &str) -> String {
        name.to_owned() + "." + extension
    }
}

#[cfg(not(android))]
use self::desktop::{read_asset, read_asset_from_path};

#[cfg(android)]
use self::android::{read_asset, read_bytes};

fn read_file_complete<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, AssetStoreError> {
    let mut file: File = File::open(&path)?;
    let mut bytes = Vec::with_capacity(file.metadata().map(|x| x.len()).unwrap_or(512) as usize);

    file.read_to_end(&mut bytes)?;

    Ok(bytes.into_boxed_slice())
}

/// The default store, which defaults
/// to the "assets" directory on desktop
/// platforms and to embedded assets on
/// Android. Should be used if you do
/// not need anything special.
#[derive(Clone, Debug)]
pub struct DefaultStore;

/// A directory store which just searches for
/// an asset in a directory.
/// Does only work on desktop.
#[derive(Clone, Debug)]
#[cfg(not(android))]
pub struct DirectoryStore {
    /// The path the assets are imported from.
    /// Note that there are subfolders, as specified
    /// in the `Asset` type.
    pub path: PathBuf,
}

/// An asset store that
/// loads assets from a ZIP
/// file.
#[derive(Clone, Debug)]
pub struct ZipStore {
    files: HashMap<String, Box<[u8]>>,
}

impl DirectoryStore {
    /// Creates a new `DirectoryStore` which will read from
    /// the specified directory.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        DirectoryStore { path: path.as_ref().into() }
    }
}

impl AssetStore for DefaultStore {
    fn read_asset<T, F>(&self, name: &str, format: &F) -> Result<Box<[u8]>, AssetStoreError>
        where F: AssetFormat,
              T: Asset
    {
        read_asset::<T, _>(name, format)
    }
}

#[cfg(not(android))]
impl AssetStore for DirectoryStore {
    fn read_asset<T, F>(&self, name: &str, format: &F) -> Result<Box<[u8]>, AssetStoreError>
        where F: AssetFormat,
              T: Asset
    {
        let path = self.path.join(T::category()).join(name);

        println!("path: {:?}", path);

        read_asset_from_path(&path, format)
    }
}

impl ZipStore {
    /// Reads a zip file from a stream
    /// and returns a ZipStore.
    pub fn from<T>(stream: T) -> Result<Self, IoError>
        where T: Read + Seek
    {
        use zip::read::ZipFile;
        use zip::result::ZipError;

        let map_zip_err = |x: ZipError| {
            let e: IoError = match x {
                    ZipError::Io(x) => x.kind(),
                    ZipError::FileNotFound => ErrorKind::NotFound,
                    ZipError::InvalidArchive(_) => ErrorKind::InvalidData,
                    ZipError::UnsupportedArchive(_) => ErrorKind::InvalidData,
                }
                .into();

            e
        };

        let archive: Result<ZipArchive<T>, ZipError> = ZipArchive::<T>::new(stream);
        let mut archive = archive.map_err(|x| map_zip_err(x))?;

        let mut map = HashMap::with_capacity(archive.len());

        for file in 0..archive.len() {
            let mut file: ZipFile = archive.by_index(file).map_err(|x| map_zip_err(x))?;
            let mut bytes = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut bytes)?;
            map.insert(file.name().to_owned(), bytes.into_boxed_slice());
        }

        Ok(ZipStore { files: map })
    }

    /// Open a file and read its contents
    #[cfg(not(android))]
    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self, IoError> {
        let stream = BufReader::new(File::open(p)?);

        Self::from(stream)
    }

    /// Open a zip file and read its contents
    #[cfg(android)]
    pub fn open<P: AsRef<Path>>(p: P) -> Result<Self, IoError> {
        use std::io::Cursor;

        let stream = read_bytes(p.to_str().unwrap())?;
        let stream = Cursor::new(stream);

        Self::from(stream)
    }
}

impl AssetStore for ZipStore {
    fn read_asset<T, F>(&self, name: &str, format: &F) -> Result<Box<[u8]>, AssetStoreError>
        where F: AssetFormat,
              T: Asset
    {
        let mut pathbuf = PathBuf::from(T::category()).join(name);

        for extension in format.file_extensions() {
            pathbuf.set_extension(extension);
            let s = pathbuf.to_str().unwrap();

            if let Some(bytes) = self.files.get(s) {
                return Ok(bytes.clone());
            }
        }

        Err(AssetStoreError::NoSuchAsset)
    }
}
