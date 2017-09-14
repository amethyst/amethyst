pub use self::dir::Directory;


use std::error::Error;
use std::ops::Deref;

use futures::{Future, IntoFuture};

use BoxedErr;

mod dir;

/// A dynamic version of `Store`, allowing to use it as a trait object.
pub trait AnyStore: Send + Sync {
    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, BoxedErr>;

    fn load(
        &self,
        category: &str,
        id: &str,
        exts: &[&str],
    ) -> Box<Future<Item = Vec<u8>, Error = BoxedErr>>;
}

impl<T> AnyStore for T
where
    T: Store + Send + Sync,
{
    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, BoxedErr> {
        T::modified(self, category, id, ext).map_err(BoxedErr::new)
    }

    fn load(
        &self,
        category: &str,
        id: &str,
        exts: &[&str],
    ) -> Box<Future<Item = Vec<u8>, Error = BoxedErr>> {
        Box::new(
            T::load(self, category, id, exts)
                .into_future()
                .map_err(BoxedErr::new),
        )
    }
}

impl<'a, T, S> Store for T
where
    T: Deref<Target = S> + Send + Sync,
    S: AnyStore + ?Sized,
{
    type Error = BoxedErr;
    type Result = Box<Future<Item = Vec<u8>, Error = BoxedErr>>;

    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, Self::Error> {
        AnyStore::modified(self, category, id, ext)
    }

    fn load(&self, category: &str, id: &str, exts: &[&str]) -> Self::Result {
        AnyStore::load(&**self, category, id, exts)
    }
}


/// A trait for asset stores, which provides
/// methods for loading
pub trait Store {
    /// The error that may occur when calling `modified` or `load`.
    type Error: Error + Send + Sync + 'static;
    /// The result type of `load`.
    type Result: IntoFuture<Item = Vec<u8>, Error = Self::Error> + 'static;

    /// This is called to check if an asset has been modified.
    ///
    /// Returns the modification time as seconds since `UNIX_EPOCH`.
    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, Self::Error>;

    /// Loads the bytes given a category, id and extension of the asset.
    ///
    /// The id should always use `/`as separator in paths.
    fn load(&self, category: &str, id: &str, exts: &[&str]) -> Self::Result;
}
