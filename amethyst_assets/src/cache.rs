use fnv::FnvHashMap;
use parking_lot::RwLock;

use AssetSpec;

/// A basic implementation for a cache. This might be useful as the `Context` of
/// an `Asset`, so that the same asset doesn't get imported twice.
///
/// Because contexts have to be immutable, a `RwLock` is used. Therefore, all
/// operations are blocking (but shouldn't block for a long time).
pub struct Cache<T> {
    map: RwLock<FnvHashMap<AssetSpec, T>>,
}

impl<T> Cache<T>
where
    T: Clone,
{
    /// Creates a new `Cache` and initializes it with the default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Inserts an asset, locking the internal `RwLock` to get write access to the hash map.
    ///
    /// Returns the previous value in case there was any.
    pub fn insert(&self, spec: AssetSpec, asset: T) -> Option<T> {
        self.map.write().insert(spec, asset)
    }

    /// Retrieves an asset, locking the internal `RwLock` to get read access to the hash map.
    /// In case this asset has been inserted previously, it will be cloned and returned.
    /// Otherwise, you'll receive `None`.
    pub fn get(&self, spec: &AssetSpec) -> Option<T> {
        self.map.read().get(spec).map(Clone::clone)
    }

    /// Accesses a cached asset, locking the internal `RwLock` to get read access to the hash map.
    /// In case the asset exists, `f` gets called with a reference to the cached asset and this
    /// method returns `true`.
    pub fn access<O, F: FnOnce(&T) -> O>(&self, spec: &AssetSpec, f: F) -> Option<O> {
        if let Some(a) = self.map.read().get(spec) {
            Some(f(a))
        } else {
            None
        }
    }

    /// Deletes all cached values, except the ones `f` returned `true` for.
    /// May be used when you're about to clear unused assets (see `Asset::clear`).
    ///
    /// Blocks the calling thread for getting write access to the hash map.
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&AssetSpec, &mut T) -> bool,
    {
        self.map.write().retain(f);
    }

    /// Deletes all cached values after locking the `RwLock`.
    pub fn clear_all(&self) {
        self.map.write().clear();
    }
}

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Cache {
            map: Default::default(),
        }
    }
}
