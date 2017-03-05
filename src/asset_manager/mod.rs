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
pub use self::common::{DefaultStore, DirectoryStore, ZipStore};
pub use self::io::{Import, Error as ImportError};

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error as FormatError, Formatter};
use std::marker::{Send, PhantomData};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use futures::{Async, Future};
use futures_cpupool::{CpuPool, CpuFuture};

use threadpool::ThreadPool;

use engine::Context;
use ecs::{System, RunArg};

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
    assets: HashMap<String, Box<Any>>,
    data: Arc<Mutex<Vec<AssetData>>>,
    pool: Arc<ThreadPool>,
}

type LoadData = Box<Fn() -> Result<Box<Any + Send>, AssetError> + Send>;
type LoadAsset = Box<Fn(Box<Any + Send>, &mut Context) -> Box<Any> + Send>;

/// Wrapper type for a generic
/// asset data.
pub struct Submission {
    name: String,
    load_data: LoadData,
    load_asset: LoadAsset,
}

/// Wrapper type for a generic asset,
/// which data was already loaded.
pub struct AssetData {
    name: String,
    data: Box<Any + Send>,
    load_asset: LoadAsset,
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
pub struct AssetFuture<'a, T: Asset> {
    name: String,
    loader: &'a AssetLoader,
    phantom: PhantomData<T>,
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
    /// Importing the data didn't work
    /// out. Note that this does not necessarily mean
    /// that the data is invalid.
    DataError(String),
}

impl AssetLoader {
    /// Creates a new asset loader with a cpu pool
    /// using the number of cpu cores for the number of threads.
    pub fn new(pool: Arc<ThreadPool>) -> Self {
        AssetLoader {
            assets: HashMap::new(),
            data: Arc::new(Mutex::new(Vec::new())),
            pool: pool,
        }
    }

    fn spawn(&self, sub: Submission) {
        let data = self.data.clone();
        self.pool.execute(move || Self::execute(sub, data));
    }

    fn execute(sub: Submission, data: Arc<Mutex<Vec<AssetData>>>) {}

    /// Load an asset on the calling thread.
    /// If it is not a very small asset, you
    /// should use `load` or `load_default`
    /// instead.
    pub fn load_now<T, S, F>(store: &S,
                             name: &str,
                             format: F,
                             context: &mut Context)
                             -> Result<T, AssetError>
        where T: Asset,
              S: AssetStore,
              F: AssetFormat + Import<T::Data>
    {
        let data = Self::load_data::<T, S, F>(store, name, format)?;
        T::from_data(data, context).map_err(|x| AssetError::DataError(format!("{:?}", x)))
    }

    /// Loads just the data for some asset (blocking).
    ///
    /// Try to use `load` or `load_default` instead, unless
    /// it's a very small asset.
    pub fn load_data<T, S, F>(store: &S, name: &str, format: F) -> Result<T::Data, AssetError>
        where T: Asset,
              S: AssetStore,
              F: AssetFormat + Import<T::Data>
    {
        let bytes = store.read_asset::<T, F>(name)?;
        format.import(bytes).map_err(|x| AssetError::ImportError(x))
    }

    /// Load the data using one of the threads from the
    /// cpu pool, returning an `AssetFuture`.
    ///
    /// If you already have the asset data and you just want to
    /// import it, use `MyAsset::from_data(data, context)`.
    pub fn load<T, S, F>(&self, store: &S, name: &str, format: F) -> AssetFuture<T>
        where T: Asset + 'static,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              S: AssetStore + Clone + Send + Sync + 'static,
              F: AssetFormat + Import<T::Data> + Send + 'static
    {
        let store: S = store.clone();
        let name = name.to_string();

        let sub = Submission {
            name: name,
            load_data: Box::new(move || {
                let data = Self::load_data::<T, S, F>(&store, &name.clone(), format)?;
                let data_boxed = Box::new(data) as Box<Any + Send>;
                Ok(data_boxed)
            }),
            load_asset: Box::new(|x, context| {
                debug_assert!(x.is::<T>());

                let data: Box<T::Data> = unsafe { ::std::mem::transmute(x) };
                let data = *data;

                let asset = T::from_data(data, context);

                Box::new(asset) as Box<Any>
            }),
        };

        self.spawn(sub);

        AssetFuture {
            name: name,
            loader: self,
            phantom: PhantomData,
        }
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
        where T: Asset + 'static,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              F: AssetFormat + Import<T::Data> + Send + 'static
    {
        let store = DefaultStore;
        self.load(&store, name, format)
    }
}

impl From<AssetStoreError> for AssetError {
    fn from(e: AssetStoreError) -> Self {
        AssetError::StoreError(e)
    }
}

impl Display for AssetError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        match *self {
            AssetError::StoreError(ref x) => write!(f, "IO Error: {}", x),
            AssetError::ImportError(ref x) => write!(f, "Import Error: {}", x),
            AssetError::DataError(ref x) => {
                write!(f, "Error when instantiating asset from data: {}", x)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::default::Default;

    use gfx_device::video_init;

    #[derive(Clone)]
    struct FakeStore {
        files: HashMap<String, Box<[u8]>>,
    }

    impl AssetStore for FakeStore {
        fn read_asset<T, F>(&self, name: &str) -> Result<Box<[u8]>, AssetStoreError>
            where T: Asset,
                  F: AssetFormat
        {
            self.files
                .get(&(T::category().to_string() + "/" + name))
                .ok_or(AssetStoreError::NoSuchAsset)
                .map(|x| x.clone())
        }
    }

    fn create_context() -> Context {
        let (_, factory, _) = video_init(Default::default());
        Context::new(factory)
    }

    #[test]
    #[ignore]
    fn test_mesh_texture() {
        use self::formats::Obj;
        use ecs::components::Mesh;
        use ecs::components::Texture;

        let loader = AssetLoader::new();
        let mut store = FakeStore { files: HashMap::new() };

        store.files.insert("meshes/foo".to_string(),
                           "
v 1.0 2.0 3.0
v 4.0 5.0 6.0
v 7.0 8.0 9.0

vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0

vn 1.0 0.0 0.0

f 1/1/1 2/2/1 3/3/1
"
                               .as_bytes()
                               .to_owned()
                               .into_boxed_slice());

        let mut context = create_context();
        let mesh = loader.load::<Mesh, _, _>(&store, "foo", Obj);
        mesh.finish(&mut context).unwrap();
        Texture::from_color([1.0, 0.0, 1.0, 1.0]);
    }
}
