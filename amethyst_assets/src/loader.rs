use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;

use fnv::FnvHashMap;
use futures::{Future, IntoFuture, Poll, Async};
use futures::sync::oneshot::{channel, Receiver};
use rayon::ThreadPool;
use specs::{Component, DenseVecStorage};

use asset::AssetSpec;

use store::AnyStore;
use {Allocator, Asset, BoxedErr, Context, Directory, Format, AssetError, LoadError, Store};

enum AssetFutureInner<A, F, S>
    where A: Asset
{
    Prepared(Receiver<Result<A::Result, LoadError<A::Error, F, S>>>),
    Received(<A::Result as IntoFuture>::Future),
    Final(<Result<A, LoadError<A::Error, F, S>> as IntoFuture>::Future),
}

/// Represents a future value of an asset. This future may be
/// added to the ECS world, where the responsible system can poll it and merge
/// it into the persistent storage once it is `Ready`.
pub struct AssetFuture<A, F, S>
    where A: Asset
{
    spec: AssetSpec,
    inner: AssetFutureInner<A, F, S>,
}

impl<A, F, S> AssetFuture<A, F, S>
    where A: Asset
{
    fn new(spec: AssetSpec, asset: A::Result) -> Self {
        AssetFuture {
            spec: spec,
            inner: AssetFutureInner::Received(asset.into_future())
        }
    }

    fn ok(spec: AssetSpec, asset: A) -> Self {
        AssetFuture {
            spec: spec,
            inner: AssetFutureInner::Final(Ok(asset).into_future())
        }
    }

    fn error(error: AssetError<A::Error, F, S>) -> Self {
        let AssetError { asset, error } = error;
        AssetFuture {
            spec: asset,
            inner: AssetFutureInner::Final(Err(error).into_future())
        }
    }
    
    fn spawn<U>(spec: AssetSpec, pool: &ThreadPool, f: U) -> Self
        where U: FnOnce() -> Result<A::Result, LoadError<A::Error, F, S>> + Send + 'static,
              A::Result: Send + 'static,
              A::Error: Send + 'static,
              F: Send + 'static,
              S: Send + 'static,
    {
        let (send, recv) = channel();

        pool.spawn(move || {
            send.send(f());
        });

        AssetFuture {
            spec: spec,
            inner: AssetFutureInner::Prepared(recv)
        }
    }
}

impl<A, F, S> Future for AssetFuture<A, F, S>
    where A: Asset
{
    type Item = A;
    type Error = AssetError<A::Error, F, S>;

    fn poll(&mut self) -> Poll<A, AssetError<A::Error, F, S>> {
        let AssetFuture { ref mut spec, ref mut inner } = *self;
        let res = match *inner {
            AssetFutureInner::Prepared(ref mut recv) => {
                match recv.poll().expect("Sender was dropped") {
                    Async::Ready(Ok(res)) => {
                        let mut res = res.into_future();
                        match res.poll() {
                            Ok(Async::Ready(asset)) => return Ok(Async::Ready(asset)),
                            Ok(Async::NotReady) => res,
                            Err(err) => return Err(AssetError::new(spec.clone(), LoadError::AssetError(err))),
                        }
                    },
                    Async::Ready(Err(err)) => return Err(AssetError::new(spec.clone(), err)),
                    Async::NotReady => return Ok(Async::NotReady),
                }
            }
            AssetFutureInner::Received(ref mut res) =>
                return res.poll().map_err(|err| AssetError::new(spec.clone(), LoadError::AssetError(err))),
            AssetFutureInner::Final(ref mut res) =>
                return res.poll().map_err(|err| AssetError::new(spec.clone(), err)),
        };

        *inner = AssetFutureInner::Received(res);
        Ok(Async::NotReady)
    }
}

impl<A, F, S> Component for AssetFuture<A, F, S>
    where A: Asset + Component + Send + Sync,
          A::Error: Send + Sync,
          A::Result: Send,
          <A::Result as IntoFuture>::Future: Send + Sync,
          F: Send + Sync + 'static,
          S: Send + Sync + 'static
{
    type Storage = DenseVecStorage<Self>;
}


/// The asset loader, holding the contexts,
/// the default (directory) store and a reference to the
/// `ThreadPool`.
pub struct Loader {
    contexts: FnvHashMap<TypeId, Box<Any>>,
    directory: Arc<AnyStore>,
    pool: Arc<ThreadPool>,
    stores: FnvHashMap<String, Arc<AnyStore>>,
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
            stores: Default::default(),
        }
    }

    /// Adds a store which can later be loaded from by supplying the same `name`
    /// to `load_from`.
    pub fn add_store<I, S>(&mut self, name: I, store: S)
        where I: Into<String>,
              S: Store + Send + Sync + 'static
    {
        self.stores.insert(name.into(), Arc::new(store));
    }

    /// Registers an asset and inserts a context into the map.
    pub fn register<A, C>(&mut self, context: C)
        where A: Asset + 'static,
              C: Context<Asset=A> + 'static,
    {
        self.contexts
            .insert(TypeId::of::<A>(), Box::new(Arc::new(context)));
    }

    /// Like `load_from`, but doesn't ask the cache for the asset.
    pub fn reload<A, F, N, S>(&self,
                              name: N,
                              format: F,
                              store: &S)
                              -> AssetFuture<A, F::Error, BoxedErr>
        where A: Asset + Send + 'static,
              A::Context: Context + Send + Sync + 'static,
              A::Error: Send + 'static,
              A::Result: Send + 'static,
              F: Format<Data = A::Data> + Send + 'static,
              F::Error: Send + 'static,
              N: Into<String>,
              S: Eq + Hash + ?Sized,
              String: Borrow<S>
    {
        let context = self.context::<A::Context>();

        reload_asset_future::<A, F, N, _>(context.clone(),
                                          format,
                                          name,
                                          self.store(store),
                                          &*self.pool)
    }

    /// Loads an asset with a given format from the default (directory) store.
    /// If you want to load from a custom source instead, use `load_from`.
    ///
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    pub fn load<A, F, N>(&self,
                         id: N,
                         format: F)
                         -> AssetFuture<A, F::Error, BoxedErr>
        where A: Asset + Send + 'static,
              A::Context: Context + Send + Sync + 'static,
              A::Error: Send + 'static,
              A::Result: Send + 'static,
              F: Format<Data = A::Data> + Send + 'static,
              F::Error: Send + 'static,
              N: Into<String>
    {
        self.load_from::<A, F, _, _>(id, format, "")
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
                                 store: &S)
                                 -> AssetFuture<A, F::Error, BoxedErr>
        where A: Asset + Send + 'static,
              A::Context: Context + Send + Sync + 'static,
              A::Error: Send + 'static,
              A::Result: Send + 'static,
              F: Format<Data = A::Data> + Send + 'static,
              F::Error: Send + 'static,
              N: Into<String>,
              S: AsRef<str> + Eq + Hash + ?Sized,
              String: Borrow<S>
    {
        let context = self.context::<A::Context>();
        let store = match store.as_ref() {
            "" => self.directory.clone(),
            _ => self.store(store),
        };

        load_asset_future::<A, F, N, _>(context.clone(), format, name, store, &*self.pool)
    }

    fn context<C>(&self) -> &Arc<C>
        where C: Context + 'static,
              C::Asset: 'static
    {
        let context = self.contexts
            .get(&TypeId::of::<C::Asset>())
            .expect("Assets need to be registered with `Loader::register`.");

        context.downcast_ref().unwrap()
    }

    fn store<S>(&self, store: &S) -> Arc<AnyStore>
        where S: Eq + Hash + ?Sized,
              String: Borrow<S>
    {
        self.stores
            .get(&store)
            .expect("No such store. Maybe you forgot to add it with `Loader::add_store`?")
            .clone()
    }
}

/// Loads an asset with a given context, format, specifier and storage right now.
pub fn load_asset<A, F, N, S>(context: &A::Context,
                              format: &F,
                              name: N,
                              storage: &S)
                              -> AssetFuture<A, F::Error, S::Error>
    where A: Asset,
          A::Context: Context,
          F: Format<Data = A::Data>,
          N: Into<String>,
          S: Store
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    match context.retrieve(&spec) {
        Some(asset) => AssetFuture::new(spec, asset),
        None => {
            match load_asset_inner::<A, F, S>(context, format, &spec, storage) {
                Ok(res) => AssetFuture::new(spec, res),
                Err(err) => AssetFuture::error(AssetError::new(spec, err)),
            }
        }
    }
}

/// Loads an asset with a given context, format, specifier and storage right now.
/// Note that this method does not ask for a cached version of the asset, but just
/// reloads the asset.
pub fn reload_asset<A, F, N, S>(context: &A::Context,
                                format: &F,
                                name: N,
                                storage: &S)
                                -> AssetFuture<A, F::Error, S::Error>
    where A: Asset,
          A::Context: Context,
          F: Format<Data = A::Data>,
          N: Into<String>,
          S: Store
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    match load_asset_inner::<A, F, S>(context, format, &spec, storage) {
        Ok(res) => AssetFuture::new(spec, res),
        Err(err) => AssetFuture::error(AssetError::new(spec, err)),
    }
}

/// Like `load_asset`, but loads the asset on a worker thread and returns
/// an `AssetFuture` immediately.
pub fn load_asset_future<A, F, N, S>(context: Arc<A::Context>,
                                     format: F,
                                     name: N,
                                     storage: S,
                                     thread_pool: &ThreadPool)
                                     -> AssetFuture<A, F::Error, S::Error>
    where A: Asset + Send + 'static,
          A::Context: Context + Send + Sync + 'static,
          A::Error: Send + 'static,
          A::Result: Send + 'static,
          F: Format<Data = A::Data> + Send + 'static,
          F::Error: Send + 'static,
          N: Into<String>,
          S: Store + Send + Sync + 'static
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    match context.retrieve(&spec) {
        Some(asset) => AssetFuture::new(spec, asset),
        None => {
            AssetFuture::spawn(spec.clone(), thread_pool, move || load_asset_inner::<A, F, S>(&context, &format, &spec, &storage))
        }
    }
}

/// Like `reload_asset`, but loads the asset on a worker thread and returns
/// an `AssetFuture` immediately.
pub fn reload_asset_future<A, F, N, S>(context: Arc<A::Context>,
                                       format: F,
                                       name: N,
                                       storage: S,
                                       thread_pool: &ThreadPool)
                                       -> AssetFuture<A, F::Error, S::Error>
    where A::Context: Context + Send + Sync + 'static,
          A: Asset + Send + 'static,
          A::Error: Send + 'static,
          A::Result: Send + 'static,
          F: Format<Data = A::Data> + Send + 'static,
          F::Error: Send + 'static,
          N: Into<String>,
          S: Store + Send + Sync + 'static
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    AssetFuture::spawn(spec.clone(), thread_pool, move || load_asset_inner::<A, F, S>(&context, &format, &spec, &storage))
}

fn load_asset_inner<A, F, S>(context: &A::Context,
                             format: &F,
                             spec: &AssetSpec,
                             store: &S)
                             -> Result<A::Result, LoadError<A::Error, F::Error, S::Error>>
    where A: Asset,
          F: Format<Data = A::Data>,
          S: Store
{
    let bytes = store
        .load(context.category(), &spec.name, spec.ext)
        .map_err(LoadError::StorageError)?;
    let data = format.parse(bytes).map_err(LoadError::FormatError)?;
    Ok(context.create_asset(data))
}

