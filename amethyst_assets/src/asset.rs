use std::error::Error;

use fnv::FnvHashMap;
use futures::{Future, IntoFuture};
use parking_lot::RwLock;
use rayon::ThreadPool;

use {BoxedErr, StoreId};

/// One of the three core traits of this crate.
///
/// You want to implement this for every type of asset like
///
/// * `Mesh`
/// * `Texture`
/// * `Terrain`
///
/// and so on. Now, an asset may be available in different formats.
/// That's why we have the `Data` associated type here. You can specify
/// an intermediate format here, like the vertex data for a mesh or the samples
/// for audio data.
///
/// This data is then generated by the `Format` trait.
pub trait Asset
    where
        Self: Clone + Sized + 'static,
{
    /// The `Context` type that can produce this asset
    type Context: Context<Asset=Self, Data=Self::Data> + 'static;
    /// The `Data` type the asset can be created from.
    type Data;

    /// Pushes an update to a queue.
    /// The updated version will be applied by calling
    /// `update`.
    ///
    /// **Note:** The updated version only gets pushed on a single
    /// asset.
    fn push_update(&self, updated: Self);

    /// Applies a previously pushed update.
    fn update(&mut self);

    /// Returns `true` if another asset points to the same
    /// internal data.
    ///
    /// In case this asset wraps an `Arc`, this
    /// would be implemented with `Arc::strong_count(pointer) > 1`.
    fn is_shared(&self) -> bool;
}

pub type AssetFuture<A> = Box<Future<Item=A, Error=BoxedErr>>;

/// A specifier for an asset, uniquely identifying it by
///
/// * the extension (the format it was provided in)
/// * its name
/// * the storage it was loaded from
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetSpec {
    /// The extension of this asset
    pub ext: &'static str,
    /// The name of this asset.
    pub name: String,
    /// Unique identifier indicating the Storage from which the asset was loaded.
    pub store: StoreId,
}

impl AssetSpec {
    /// Creates a new asset specifier from the given parameters.
    pub fn new(name: String, ext: &'static str, store: StoreId) -> Self {
        AssetSpec { ext, name, store }
    }
}

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
    pub fn access<F: FnOnce(&T)>(&self, spec: &AssetSpec, f: F) -> bool {
        if let Some(a) = self.map.read().get(spec) {
            f(a);

            true
        } else {
            false
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
        Cache { map: Default::default() }
    }
}

/// The context type which manages assets of one type.
/// It is responsible for caching
pub trait Context {
    /// The asset type this context can produce.
    type Asset: Asset;
    /// The `Data` type the asset can be created from.
    type Data;
    /// The error that may be returned from `create_asset`.
    type Error: Error + Send + Sync;
    /// The result type for loading an asset. This can also be a future
    /// (or anything that implements `IntoFuture`).
    type Result: IntoFuture<Item=Self::Asset, Error=Self::Error>;

    /// A small keyword for which category these assets belongs to.
    ///
    /// ## Examples
    ///
    /// * `"mesh"` for `Mesh`
    /// * `"data"` for `Level`
    ///
    /// The storage may use this information, to e.g. search the identically-named
    /// subfolder.
    fn category(&self) -> &str;

    /// Provides the conversion from the data format to the actual asset.
    fn create_asset(&self, data: Self::Data, pool: &ThreadPool) -> Self::Result;

    /// Notifies about an asset load. This is can be used to cache the asset.
    /// To return a cached asset, see the `retrieve` function.
    fn cache(&self, _spec: AssetSpec, _asset: AssetFuture<Self::Asset>) {}

    /// Returns `Some` cached value if possible, otherwise `None`.
    ///
    /// For a basic implementation of a cache, please take a look at the `Cache` type.
    fn retrieve(&self, _spec: &AssetSpec) -> Option<AssetFuture<Self::Asset>> {
        None
    }

    /// Updates an asset after it's been reloaded.
    ///
    /// This usually just puts the new asset into a queue;
    /// the actual update happens by calling `update` on the
    /// asset.
    fn update(&self, spec: &AssetSpec, asset: Self::Asset);

    /// Gives a hint that several assets may have been released recently.
    ///
    /// This is useful if your assets are reference counted, because you are
    /// now able to remove unique assets from the cache, leaving the shared
    /// ones there.
    fn clear(&self) {}

    /// Request for clearing the whole cache.
    fn clear_all(&self) {}
}

/// A format, providing a conversion from bytes to asset data, which is then
/// in turn accepted by `Asset::from_data`. Examples for formats are
/// `Png`, `Obj` and `Wave`.
pub trait Format
    where
        Self: Sized,
{
    /// The data type this format is able to load.
    type Data;
    /// The error that may be returned from `Format::parse`.
    type Error: Error + Send + Sync;
    /// The result of the `parse` method. Can be anything that implements
    /// `IntoFuture`.
    type Result: IntoFuture<Item=Self::Data, Error=Self::Error>;

    /// Returns the extension (without `.`).
    ///
    /// ## Examples
    ///
    /// * `"png"`
    /// * `"obj"`
    /// * `"wav"`
    fn extension() -> &'static str;

    /// Reads the given bytes and produces asset data.
    fn parse(&self, bytes: Vec<u8>) -> Self::Result;
}
