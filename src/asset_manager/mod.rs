//! Asset manager used to load assets (like `Mesh`es and `Texture`s).
//!
//! For how to implement an asset yourself, see the `Asset` trait.
//!
//! If you just want to load it, look at `AssetLoader`.

pub mod formats;

mod asset;
mod common;
mod io;

pub use self::asset::{Asset, AssetFormat, AssetStore, AssetStoreError};
pub use self::common::{DefaultStore, DirectoryStore, ZipStore};
pub use self::io::{Import, Error as ImportError};

use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Display, Error as FormatError, Formatter};
use std::marker::Send;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use threadpool::ThreadPool;

use engine::Context;

/// A parallel asset loader.
/// It has access to a shared threadpool on
/// which the data is loaded.
///
/// The asset itself will be created on the main
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
/// loader.load(my_store, "my_asset", MyFormat);
/// // Later
/// let asset_option: MyAsset = loader.asset("my_asset").unwrap();
/// ```
pub struct AssetLoader {
    assets: HashMap<String, Box<Any>>,
    data: Arc<Mutex<Vec<AssetData>>>,
    error_handler: Box<ErrorHandler>,
    panicked: Arc<AtomicBool>,
    pool: Arc<ThreadPool>,
}

type LoadData = Box<Fn() -> Result<Box<Any + Send>, AssetError> + Send>;
type LoadAsset = Box<Fn(Box<Any + Send>, &mut Context) -> Result<Box<Any>, AssetError> + Send>;

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

/// The default error handler, which panics on
/// an error.
pub struct DefaultErrorHandler;

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

/// Error handler trait.
///
/// You can provide such a handler for
/// the asset loader to handle the loading errors.
/// The default is to panic.
pub trait ErrorHandler: Send {
    /// Handle an error. Store "true" in `panicked` if the main
    /// thread should panic.
    fn handle_error(&self, name: String, e: AssetError, panicked: Arc<AtomicBool>);

    /// Clones this trait object.
    fn box_clone(&self) -> Box<ErrorHandler>;
}

impl AssetLoader {
    /// Creates a new asset loader with a cpu pool
    /// using the number of cpu cores for the number of threads.
    pub fn new(pool: Arc<ThreadPool>) -> Self {
        AssetLoader {
            assets: HashMap::new(),
            data: Arc::new(Mutex::new(Vec::new())),
            error_handler: Box::new(DefaultErrorHandler),
            panicked: Arc::new(AtomicBool::new(false)),
            pool: pool,
        }
    }

    fn spawn(&self, sub: Submission) {
        let data = self.data.clone();
        let error_handler = self.error_handler.box_clone();
        let panicked = self.panicked.clone();
        self.pool.execute(move || Self::execute(sub, data, error_handler, panicked));
    }

    fn execute(sub: Submission,
               data_vec: Arc<Mutex<Vec<AssetData>>>,
               error_handler: Box<ErrorHandler>,
               panicked: Arc<AtomicBool>) {
        let Submission { name, load_data, load_asset } = sub;

        match load_data() {
            Ok(data) => {
                let mut data_vec = data_vec.lock().unwrap();
                let asset_data = AssetData {
                    name: name,
                    data: data,
                    load_asset: load_asset,
                };
                data_vec.push(asset_data);
            }
            Err(x) => {
                error_handler.handle_error(name, x, panicked);
            }
        }
    }

    /// Set an error handler, which will be executed
    /// when an error occured.
    #[inline]
    pub fn set_error_handler(&mut self, e: Box<ErrorHandler + Send>) {
        self.error_handler = e;
    }

    /// Tries to retrieve an
    /// asset from this asset loader.
    ///
    /// An error may be returned if the asset could not be loaded.
    /// Note that such an error is only returned the first time you
    /// call this method.
    ///
    /// If `Ok(None)` is returned, the asset might not be submitted
    /// yet, has another type or it hasn't finished.
    #[inline]
    pub fn asset<T>(&self, name: &str) -> Option<&T>
        where T: Asset + 'static
    {
        self.assets.get(&name.to_owned()).map(|x| x.downcast_ref().unwrap())
    }

    /// Load an asset on the calling thread.
    /// If it is not a very small asset, you
    /// should use `load` or `load_default`
    /// instead.
    pub fn load_now<'a, T, S, F>(&'a mut self,
                                 store: &'a S,
                                 name: &'a str,
                                 format: F,
                                 context: &'a mut Context)
                                 -> Result<T, AssetError>
        where T: Asset + Clone + 'static,
              S: AssetStore,
              F: AssetFormat + Import<T::Data>
    {
        let data = Self::load_data::<T, S, F>(store, name, format)?;
        let asset = T::from_data(data, context)
            .map_err(|x| AssetError::DataError(format!("{:?}", x)));
        let name = name.to_owned();
        let asset = asset?;
        self.assets.insert(name.clone(), Box::new(asset.clone()) as Box<Any>);
        Ok(asset)
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

    /// Submit an asset to load, using a shared threadpool.
    ///
    /// If you already have the asset data and you just want to
    /// import it, use `load_from_data`.
    pub fn load<T, S, F>(&self, store: &S, name: &str, format: F)
        where T: Asset + 'static,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              S: AssetStore + Clone + Send + Sync + 'static,
              F: AssetFormat + Import<T::Data> + Clone + Send + 'static
    {
        let name = name.to_string();

        if self.assets.contains_key(&name) {
            // There's nothing to do

            return;
        }

        let store: S = store.clone();
        let name_closure = name.clone();

        let sub = Submission {
            name: name.clone(),
            load_data: Box::new(move || {
                let data = Self::load_data::<T, S, F>(&store, &name_closure, format.clone())?;
                let data_boxed = Box::new(data) as Box<Any + Send>;
                Ok(data_boxed)
            }),
            load_asset: Box::new(|x, context| {
                let data: T::Data = *x.downcast::<T::Data>().unwrap();
                let asset = T::from_data(data, context).map_err(|x| AssetError::DataError(format!("{:?}", x)))?;

                Ok(Box::new(asset) as Box<Any>)
            }),
        };

        self.spawn(sub);
    }

    /// Load multiple assets with the same format.
    #[inline]
    pub fn load_all<T, S, F>(&self, store: &S, names: &[&str], format: F)
        where T: Asset + 'static,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              S: AssetStore + Clone + Send + Sync + 'static,
              F: AssetFormat + Import<T::Data> + Clone + Send + 'static
    {
        for name in names {
            self.load::<T, S, F>(store, name, format.clone());
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
    pub fn load_default<T, F>(&self, name: &str, format: F)
        where T: Asset + 'static,
              T::Data: Send + 'static,
              T::Error: Send + 'static,
              F: AssetFormat + Import<T::Data> + Clone + Send + 'static
    {
        let store = DefaultStore;
        self.load::<T, _, F>(&store, name, format);
    }

    /// Returns the number of assets
    /// which were loaded by this asset loader.
    #[inline]
    pub fn num_assets(&self) -> usize {
        self.assets.len()
    }

    fn panic_if_needed(&self) {
        if self.panicked.load(Ordering::Relaxed) {
            panic!("Propagating panic of asset loader thread to main thread");
        }
    }

    /// Process a number of asset datasets
    /// as long as `as_long` returns true.
    ///
    /// Intended to be used from the internals
    /// of the engine to allow loading assets
    /// as long as the wished frame time wasn't reached
    /// yet.
    pub fn process<F: FnMut() -> bool>(&mut self, context: &mut Context, mut as_long: F) {
        self.panic_if_needed();

        let mut data_vec = self.data.lock().unwrap();

        loop {
            let asset_data = match data_vec.pop() {
                Some(x) => x,
                None => break,
            };

            let AssetData { name, data, load_asset } = asset_data;

            let res = load_asset(data, context);

            match res {
                Ok(x) => {
                    self.assets.insert(name, x);
                }
                Err(x) => {
                    self.error_handler.handle_error(name, x, self.panicked.clone());
                    self.panic_if_needed()
                }
            }

            if !as_long() {
                break;
            }
        }
    }
}

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, name: String, error: AssetError, panicked: Arc<AtomicBool>) {
        panicked.store(true, Ordering::Relaxed);
        panic!("Failed to load asset \"{}\": {}", name, error);
    }

    fn box_clone(&self) -> Box<ErrorHandler> {
        Box::new(DefaultErrorHandler) as Box<ErrorHandler>
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
