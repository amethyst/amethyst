pub use self::dir::Directory;

use BoxedErr;

mod dir;

/// A trait for asset sources, which provides
/// methods for loading bytes.
pub trait Source: Send + Sync + 'static {
    /// This is called to check if an asset has been modified.
    ///
    /// Returns the modification time as seconds since `UNIX_EPOCH`.
    fn modified(&self, path: &str) -> Result<u64, BoxedErr>;

    /// Loads the bytes given a path.
    ///
    /// The id should always use `/` as separator in paths.
    fn load(&self, path: &str) -> Result<Vec<u8>, BoxedErr>;
}
