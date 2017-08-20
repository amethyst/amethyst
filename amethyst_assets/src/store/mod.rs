pub use self::dir::Directory;

use std::error::Error;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{Future, IntoFuture};

use BoxedErr;

mod dir;

/// An `Allocator`, holding a counter for producing unique IDs for the stores.
#[derive(Debug, Default)]
pub struct Allocator {
    store_count: AtomicUsize,
}

impl Allocator {
    /// Creates a new `Allocator`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Produces a new store id.
    pub fn next_store_id(&self) -> StoreId {
        StoreId(self.store_count.fetch_add(1, Ordering::Relaxed))
    }
}

/// A dynamic version of `Store`, allowing to use it as a trait object.
pub trait AnyStore: Send + Sync + 'static {
    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, BoxedErr>;

    fn load(&self, category: &str, id: &str, ext: &str) -> Box<Future<Item = Vec<u8>, Error = BoxedErr>>;

    fn store_id(&self) -> StoreId;
}

impl<T> AnyStore for T
    where T: Store + Send + Sync + 'static
{
    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, BoxedErr> {
        T::modified(self, category, id, ext)
            .map_err(BoxedErr::new)
    }

    fn load(&self, category: &str, id: &str, ext: &str) -> Box<Future<Item = Vec<u8>, Error = BoxedErr>> {
        Box::new(T::load(self, category, id, ext).into_future().map_err(BoxedErr::new))
    }

    fn store_id(&self) -> StoreId {
        T::store_id(self)
    }
}

impl<T, S> Store for T
    where T: Deref<Target=S>,
          S: AnyStore + ?Sized
{
    type Error = BoxedErr;
    type Result = Box<Future<Item = Vec<u8>, Error = BoxedErr>>;

    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, Self::Error> {
        S::modified(self, category, id, ext)
    }

    fn load(&self, category: &str, id: &str, ext: &str) -> Self::Result {
        S::load(self, category, id, ext)
    }

    fn store_id(&self) -> StoreId {
        S::store_id(self)
    }
}


/// A trait for asset stores, which provides
/// methods for loading
pub trait Store {
    /// The error that may occur when calling `modified` or `load`.
    type Error: Error + Send + Sync + 'static;
    /// The result type of `load`.
    type Result: IntoFuture<Item = Vec<u8>, Error = Self::Error>;

    /// This is called to check if an asset has been modified.
    ///
    /// Returns the modification time as seconds since `UNIX_EPOCH`.
    fn modified(&self, category: &str, id: &str, ext: &str) -> Result<u64, Self::Error>;

    /// Loads the bytes given a category, id and extension of the asset.
    ///
    /// The id should always use `/`as separator in paths.
    fn load(&self, category: &str, id: &str, ext: &str) -> Self::Result;

    /// Returns the unique store id. You'll often want to just
    /// have such a field for your storage which is initialized using
    /// `Allocator::next_store_id`.
    fn store_id(&self) -> StoreId;
}

/// A unique store id, used to identify the storage in `AssetSpec`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StoreId(usize);

impl StoreId {
    /// Returns a copy of the internal id.
    pub fn id(&self) -> usize {
        self.0
    }
}
