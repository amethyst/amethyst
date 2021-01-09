use amethyst_error::Error;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

pub use self::dir::Directory;

mod dir;

/// A trait for asset sources, which provides
/// methods for loading bytes.
pub trait Source: Send + Sync + 'static {
    /// This is called to check if an asset has been modified.
    ///
    /// Returns the modification time as seconds since `UNIX_EPOCH`.
    fn modified(&self, path: &str) -> Result<u64, Error>;

    /// Loads the bytes given a path.
    ///
    /// The id should always use `/` as separator in paths.
    fn load(&self, path: &str) -> Result<Vec<u8>, Error>;

    /// Returns both the result of `load` and `modified` as a tuple.
    /// There's a default implementation which just calls both methods,
    /// but you may be able to provide a more optimized version yourself.
    fn load_with_metadata(&self, path: &str) -> Result<(Vec<u8>, u64), Error> {
        #[cfg(feature = "profiler")]
        profile_scope!("source_load_asset_with_metadata");

        let m = self.modified(path)?;
        let b = self.load(path)?;

        Ok((b, m))
    }
}
