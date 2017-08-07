use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;

use fnv::FnvHashMap;
use futures::{Future, Poll};
use rayon::ThreadPool;
use specs::{Component, DenseVecStorage};

use asset::AssetSpec;

use store::AnyStore;
use {Allocator, Asset, BoxedErr, Context, Directory, Format, AssetError, LoadError, Store};

/// Represents a future value of an asset. This future may be
/// added to the ECS world, where the responsible system can poll it and merge
/// it into the persistent storage once it is `Ready`.
pub struct AssetFuture<A, E> {
    inner: Option<Arc<AssetFutureCell<A, E>>>,
}

impl<A: 'static, E: 'static> AssetFuture<A, E> {
    fn spawn<F>(pool: &ThreadPool, f: F) -> Self
        where F: FnOnce() -> Result<A, E> + Send + 'static
    {
        let inner = AssetFutureCell { value: UnsafeCell::new(None) };
        let inner = Arc::new(inner);

        let cloned = inner.clone();

        pool.spawn(move || {
            let res = f();
            // This is safe because the other reference to `value` is only accessed
            // if there is just one strong reference of the `Arc`. However, this closure holds
            // one.
            unsafe {
                *cloned.value.get() = Some(res);
            }

            // Now, `cloned` gets dropped.
        });

        let inner = Some(inner);

        AssetFuture { inner }
    }
}

impl<A, E> Component for AssetFuture<A, E>
    where A: Component,
          E: 'static
{
    type Storage = DenseVecStorage<Self>;
}

impl<A, E> Future for AssetFuture<A, E> {
    type Item = A;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use futures::Async;

        match Arc::try_unwrap(self.inner.take().unwrap()) {
            Ok(x) => {
                // As soon as the worker thread finished executing the closure, the second
                // reference gets dropped and we enter this branch.
                // Thus, we are the only one with access to the cell.
                unsafe {
                    x.value
                        .into_inner()
                        .expect("Thread panicked")
                        .map(Async::Ready)
                }
            }
            Err(arc) => {
                self.inner = Some(arc);

                Ok(Async::NotReady)
            }
        }
    }

    fn wait(mut self) -> Result<Self::Item, Self::Error> {
        use futures::Async;

        loop {
            match self.poll() {
                Ok(Async::Ready(x)) => return Ok(x),
                Ok(Async::NotReady) => {}
                Err(x) => return Err(x),
            }
        }
    }
}

struct AssetFutureCell<A, E> {
    value: UnsafeCell<Option<Result<A, E>>>,
}

unsafe impl<A, E> Send for AssetFutureCell<A, E> {}
unsafe impl<A, E> Sync for AssetFutureCell<A, E> {}

/// The asset loader, holding the contexts,
/// the default (directory) store and a reference to the
/// `ThreadPool`.
pub struct Loader {
    contexts: FnvHashMap<TypeId, Box<Any>>,
    directory: Arc<Box<AnyStore>>,
    pool: Arc<ThreadPool>,
    stores: FnvHashMap<String, Arc<Box<AnyStore>>>,
}

impl Loader {
    /// Creates a new asset loader, initializing the directory store with the
    /// given path.
    pub fn new<P>(alloc: &Allocator, directory: P, pool: Arc<ThreadPool>) -> Self
        where P: Into<PathBuf>
    {
        Loader {
            contexts: Default::default(),
            directory: Arc::new(Box::new(Directory::new(alloc, directory))),
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
        self.stores.insert(name.into(), Arc::new(Box::new(store)));
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
                              -> AssetFuture<A, AssetError<A::Error, F::Error, BoxedErr>>
        where A: Asset + Send + 'static,
              A::Context: Context + Send + Sync + 'static,
              A::Error: Send + 'static,
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
                         -> AssetFuture<A, AssetError<A::Error, F::Error, BoxedErr>>
        where A: Asset + Send + 'static,
              A::Context: Context + Send + Sync + 'static,
              A::Error: Send + 'static,
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
                                 -> AssetFuture<A, AssetError<A::Error, F::Error, BoxedErr>>
        where A: Asset + Send + 'static,
              A::Context: Context + Send + Sync + 'static,
              A::Error: Send + 'static,
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

    fn store<S>(&self, store: &S) -> Arc<Box<AnyStore>>
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
                              -> Result<A, AssetError<A::Error, F::Error, S::Error>>
    where A: Asset,
          A::Context: Context,
          F: Format<Data = A::Data>,
          N: Into<String>,
          S: Store
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    context.retrieve(&spec)
        .map(|a| Ok(a))
        .unwrap_or_else(move || load_asset_inner(context, format, spec, storage))
        .map_err(|e| AssetError::new(AssetSpec::new(name, F::extension(), store_id), e))
}

/// Loads an asset with a given context, format, specifier and storage right now.
/// Note that this method does not ask for a cached version of the asset, but just
/// reloads the asset.
pub fn reload_asset<A, F, N, S>(context: &A::Context,
                                format: &F,
                                name: N,
                                storage: &S)
                                -> Result<A, AssetError<A::Error, F::Error, S::Error>>
    where A: Asset,
          A::Context: Context,
          F: Format<Data = A::Data>,
          N: Into<String>,
          S: Store
{
    let name = name.into();

    let store_id = storage.store_id();
    let spec = AssetSpec::new(name.clone(), F::extension(), store_id);

    load_asset_inner(context, format, spec, storage)
        .map_err(|e| AssetError::new(AssetSpec::new(name, F::extension(), store_id), e))
}

fn load_asset_inner<C, F, S>(context: &C,
                             format: &F,
                             spec: AssetSpec,
                             store: &S)
                             -> Result<C::Asset, LoadError<C::Error, F::Error, S::Error>>
    where C: Context,
          F: Format<Data = C::Data>,
          S: Store
{
    let bytes = store
        .load(context.category(), &spec.name, spec.ext)
        .map_err(LoadError::StorageError)?;
    let data = format.parse(bytes).map_err(LoadError::FormatError)?;
    let a = context.create_asset(data)
        .map_err(LoadError::AssetError)?;

    context.cache(spec, &a);

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
          A::Context: Context + Send + Sync + 'static,
          A::Error: Send + 'static,
          F: Format<Data = A::Data> + Send + 'static,
          F::Error: Send + 'static,
          N: Into<String>,
          S: Store + Send + Sync + 'static
{
    let name = name.into();

    let closure = move || load_asset::<A, F, _, S>(&*context, &format, name, &*storage);

    AssetFuture::spawn(thread_pool, closure)
}

/// Like `reload_asset`, but loads the asset on a worker thread and returns
/// an `AssetFuture` immediately.
pub fn reload_asset_future<A, F, N, S>
    (context: Arc<A::Context>,
     format: F,
     name: N,
     storage: Arc<S>,
     thread_pool: &ThreadPool)
     -> AssetFuture<A, AssetError<A::Error, F::Error, S::Error>>
    where A::Context: Context + Send + Sync + 'static,
          A: Asset + Send + 'static,
          A::Error: Send + 'static,
          F: Format<Data = A::Data> + Send + 'static,
          F::Error: Send + 'static,
          N: Into<String>,
          S: Store + Send + Sync + 'static
{
    let name = name.into();

    let closure = move || reload_asset::<A, F, _, S>(&*context, &format, name, &*storage);

    AssetFuture::spawn(thread_pool, closure)
}
