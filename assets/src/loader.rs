use std::any::{Any, TypeId};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::sync::Arc;

use fnv::FnvHashMap;
use futures::{Future, Poll};
use rayon::{RayonFuture, ThreadPool};

use {Allocator, Asset, AssetSpec, Directory, Format, AssetError, LoadError, Store};

/// Represents a future value of an asset. This future may be
/// added to the ECS world, where the responsible system can poll it and merge
/// it into the persistent storage once it is `Ready`.
pub struct AssetFuture<A, E>(RayonFuture<A, E>);

impl<A, E> Future for AssetFuture<A, E> {
    type Item = A;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }

    fn wait(self) -> Result<Self::Item, Self::Error>
        where Self: Sized
    {
        self.0.wait()
    }
}

/// The asset loader, holding the contexts,
/// the default (directory) store and a reference to the
/// `ThreadPool`.
pub struct Loader {
    contexts: FnvHashMap<TypeId, Box<Any>>,
    directory: Arc<Directory>,
    pool: Arc<ThreadPool>,
}

impl Loader {
    /// Creates a new asset loader, initializing the directory store with the
    /// given path.
    pub fn new<P>(alloc: &Allocator, directory: P, pool: Arc<ThreadPool>) -> Self
        where P: Into<PathBuf>
    {
        Loader {
            contexts: Default::default(),
            directory: Arc::new(Directory::new(alloc, directory)),
            pool: pool,
        }
    }

    /// Registers an asset and inserts a context into the map.
    pub fn register<A>(&mut self, context: A::Context)
        where A: Asset + 'static,
              A::Context: 'static
    {
        self.contexts
            .insert(TypeId::of::<A>(), Box::new(Arc::new(context)));
    }

    /// Loads an asset with a given format from the default (directory) store.
    /// If you want to load from a custom source instead, use `load_from`.
    ///
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    pub fn load<A, F, N>(&self,
                         id: N,
                         format: F)
                         -> AssetFuture<A, AssetError<A::Error, F::Error, IoError>>
        where A: Asset + Send + 'static,
              A::Context: Send + Sync + 'static,
              A::Error: Send + 'static,
              F: Format<Data = A::Data> + Send + 'static,
              F::Error: Send + 'static,
              N: Into<String>
    {
        self.load_from::<A, F, _, _>(id, format, self.directory.clone())
    }

    /// Loads an asset with a given id and format from a custom store.
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    ///
    /// # Panics
    ///
    /// Panics if the asset wasn't registered.
    pub fn load_from<A, F, N, S>(&self,
                                 name: N,
                                 format: F,
                                 store: Arc<S>)
                                 -> AssetFuture<A, AssetError<A::Error, F::Error, S::Error>>
        where A: Asset + Send + 'static,
              A::Context: Send + Sync + 'static,
              A::Error: Send + 'static,
              F: Format<Data = A::Data> + Send + 'static,
              F::Error: Send + 'static,
              N: Into<String>,
              S: Store + Send + Sync + 'static,
              S::Error: Send
    {
        let context = self.contexts
            .get(&TypeId::of::<A>())
            .expect("Assets need to be registered");

        let context: &Arc<A::Context> = context.downcast_ref().unwrap();

        load_asset_future::<A, F, N, S>(context.clone(), format, name, store, &*self.pool)
    }
}

/// Loads an asset with a given context, format, specifier and storage right now.
pub fn load_asset<A, F, N, S>(context: &A::Context,
                              format: &F,
                              name: N,
                              storage: &S)
                              -> Result<A, AssetError<A::Error, F::Error, S::Error>>
    where A: Asset,
          F: Format<Data = A::Data>,
          N: Into<String>,
          S: Store
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    A::cached(context, &spec)
        .map(|a| Ok(a))
        .unwrap_or_else(move || load_asset_inner(context, format, spec, storage))
        .map_err(|e| AssetError::new(AssetSpec::new(name, F::extension(), store_id), e))
}

fn load_asset_inner<A, F, S>(context: &A::Context,
                             format: &F,
                             spec: AssetSpec,
                             storage: &S)
                             -> Result<A, LoadError<A::Error, F::Error, S::Error>>
    where A: Asset,
          F: Format<Data = A::Data>,
          S: Store
{
    let bytes = storage
        .load(A::category(), &spec.name, spec.ext)
        .map_err(LoadError::StorageError)?;
    let data = format.parse(bytes).map_err(LoadError::FormatError)?;
    let a = Asset::from_data(data, context)
        .map_err(LoadError::AssetError)?;

    A::asset_loaded(context, spec, &a);

    Ok(a)
}

/// Like `load_asset`, but loads the asset on a worker thread and returns
/// an `AssetFuture` immediately.
pub fn load_asset_future<A, F, N, S>(context: Arc<A::Context>,
                                     format: F,
                                     name: N,
                                     storage: Arc<S>,
                                     thread_pool: &ThreadPool)
                                     -> AssetFuture<A, AssetError<A::Error, F::Error, S::Error>>
    where A: Asset + Send + 'static,
          A::Context: Send + Sync + 'static,
          A::Error: Send + 'static,
          F: Format<Data = A::Data> + Send + 'static,
          F::Error: Send + 'static,
          N: Into<String>,
          S: Store + Send + Sync + 'static,
          S::Error: Send + 'static
{
    use futures::future::lazy;

    let name = name.into();

    let closure = move || load_asset::<A, F, _, S>(&*context, &format, name, &*storage);

    AssetFuture(thread_pool.spawn_future_async(lazy(closure)))
}
