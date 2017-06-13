pub use self::dir::Directory;

use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// A trait for asset stores, which provides
/// methods for loading
pub trait Store {
    type Error: Error + 'static;
    type Location;

    /// Loads the bytes given a category, id and extension of the asset.
    ///
    /// The id should always use `/`as separator in paths.
    fn load(&self, category: &str, id: &str, ext: &str) -> Result<Vec<u8>, Self::Error>;

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
