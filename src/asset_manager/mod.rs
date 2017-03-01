//! Asset manager used to load assets (like `Mesh`es and `Texture`s).
//!
//! For how to implement an asset yourself, see the `Asset` trait.
//!
//! If you just want to load it, look at `AssetLoader` / `AssetManager`.

pub mod formats;

mod asset;
mod common;
mod io;

pub use self::asset::{Asset, AssetFormat, AssetStore, AssetStoreError};
pub use self::common::{DefaultStore, DirectoryStore};
pub use self::io::{Import, Error as ImportError};

use std::fmt::{Display, Error as FormatError, Formatter};
use std::marker::Send;

use futures::{Async, Future};
use futures_cpupool::{CpuPool, CpuFuture};

use engine::Context;

/// A parallel asset loader.
/// It contains a `CpuPool` and
/// creates `Future`s from that.
///
/// Only the `Asset::Data` will be
/// loaded in a separate thread, the
/// asset itself will be created on the main
/// thread, because often it requires thread-unsafe
/// actions (accessing OpenGL to create some buffer).
///
/// The asset loader needs three things in order to
/// load an asset:
///
/// * An asset store (`AssetStore`): Responsible for providing bytes for a name and a format
/// * The asset name: A simple `&str`
/// * The asset format (`AssetFormat`): Has to implement `Import` (create the `Asset::Data` structure) and provide the typical file extension
///
/// # Examples
///
/// ```ignore
/// use amethyst::asset_manager::AssetLoader;
///
/// let loader = AssetLoader::new();
/// let asset_future = loader.load(my_store, "my_asset", MyFormat);
/// ```
pub struct AssetLoader {
    cpupool: CpuPool,
}

/// An asset future which means
/// "This will be available in the future".
///
/// #### Why isn't it available immediately?
///
/// The reason is that assets are loaded in
/// separate threads, without blocking the
/// main thread. Once they're finished,
/// you can access it.
///
/// As soon as you need the asset, you call
/// `finish` on this future which will return
/// the loaded data and create an asset from that
/// data. If it hasn't yet finished, it will block
/// the calling thread.
pub struct AssetFuture<T: Asset> {
    inner: CpuFuture<T::Data, AssetError>,
}

/// The error that can occurr when trying
/// to import asset data from a store.
#[derive(Debug)]
pub enum AssetError {
    /// Occurs if the `AssetStore` could not load
    /// the asset. See `AssetStoreInfo` for details.
    StoreError(AssetStoreError),
    /// Raised if the data is in an invalid format
    /// or there was an io error.
    ImportError(ImportError),
}

/// An error type which may be return when using
/// `AssetFuture::finish`.
#[derive(Debug)]
pub enum FinishError<T: Asset> {
    /// There was an `AssetError`.
    Asset(AssetError),
    /// The asset could not be instantiated from
    /// the data.
    Finish(T::Error), // TODO: Find better names
}

impl AssetLoader {
    /// Creates a new asset loader with a cpu pool
    /// using the number of cpu cores for the number of threads.
    pub fn new() -> Self {
        AssetLoader { cpupool: CpuPool::new_num_cpus() }
    }

    /// Loads just the data for some asset (blocking).
    pub fn load_data<T, D, S, F>(store: &S, name: &str, format: F) -> Result<D, AssetError>
        where T: Asset<Data = D>,
              S: AssetStore,
              F: AssetFormat + Import<D>
    {
        let bytes = store.read_asset::<T, _>(name, &format)?;
        format.import(bytes).map_err(|x| AssetError::ImportError(x))
    }

    /// Load the data using one of the threads from the
    /// cpu pool, returning an `AssetFuture`.
    ///
    /// If you already have the asset data and you just want to
    /// import it, use `MyAsset::from_data(data, context)`.
    pub fn load<T, S, F>(&self, store: &S, name: &str, format: F) -> AssetFuture<T>
        where T: Asset,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              S: AssetStore + Clone + Send + Sync + 'static,
              F: AssetFormat + Import<T::Data> + Send + 'static
    {
        let store: S = store.clone();
        let name = name.to_string();

        let cpu_future: CpuFuture<T::Data, _> = self.cpupool
            .spawn_fn(move || Self::load_data::<T, _, _, _>(&store, &name, format));

        AssetFuture { inner: cpu_future }
    }

    /// Loads an asset from
    /// the `DefaultStore`. You should use
    /// use this if possible because it is cross
    /// platform.
    ///
    /// On desktop, it just loads an asset from
    /// the "assets" folder, on android it will
    /// load it from the embedded assets.
    pub fn load_default<T, F>(&self, name: &str, format: F) -> AssetFuture<T>
        where T: Asset,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              F: AssetFormat + Import<T::Data> + Send + 'static
    {
        let store = DefaultStore;
        self.load(&store, name, format)
    }
}

impl<T: Asset + Send> AssetFuture<T> {
    /// This blocks the current thread until the data
    /// is imported (if it isn't already). After that,
    /// it'll do things which have to be done on the
    /// main thread, most likely uploading data to
    /// the graphics card.
    ///
    /// # Examples
    ///
    /// You would use it like this:
    ///
    /// ```ignore
    /// # use amethyst::asset_manager::AssetLoader;
    ///
    /// let tree = loader.load_default("tree", MyFormat);
    ///
    /// // Display loading screen
    ///
    /// let tree = tree.finish(&mut context);
    /// ```
    pub fn finish(self, context: &mut Context) -> Result<T, FinishError<T>>
        where Self: Future<Item = T::Data, Error = AssetError>
    {
        let data = self.wait()?;
        T::from_data(data, context).map_err(|x| FinishError::Finish(x))
    }
}

impl<T: Asset + 'static> Future for AssetFuture<T>
    where T::Data: Send
{
    type Item = T::Data;
    type Error = AssetError;

    fn poll(&mut self) -> Result<Async<T::Data>, AssetError> {
        self.inner.poll()
    }
}

impl From<AssetStoreError> for AssetError {
    fn from(e: AssetStoreError) -> Self {
        AssetError::StoreError(e)
    }
}

impl<T: Asset> From<AssetError> for FinishError<T> {
    fn from(e: AssetError) -> Self {
        FinishError::Asset(e)
    }
}

impl<T: Asset> Display for FinishError<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &FinishError::Asset(ref x) => write!(f, "Error when loading asset data: {}", x),
            &FinishError::Finish(ref x) => {
                write!(f, "Error when instantiating asset from data: {:?}", x)
            }
        }
    }
}

impl Display for AssetError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &AssetError::StoreError(ref x) => write!(f, "IO Error: {}", x),
            &AssetError::ImportError(ref x) => write!(f, "Import Error: {}", x),
        }
    }
}
