use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fnv::FnvHashMap;
use futures::{Async, Future, IntoFuture, Poll};
use futures::sync::oneshot::{channel, Receiver};
use rayon::ThreadPool;

use {Asset, AssetError, AssetFuture, BoxedErr, Context, Directory, Format, LoadError, Store};
use asset::AssetSpec;
use store::AnyStore;

/// Represents a future value of an asset. This future may be
/// added to the ECS world, where the responsible system can poll it and merge
/// it into the persistent storage once it is `Ready`.
pub struct SpawnedFuture<A, E>(Receiver<Result<A, E>>);

impl<A: 'static, E: 'static> SpawnedFuture<A, E> {
    /// Creates a SpawnedFuture and starts processing it.
    pub fn spawn<F>(pool: &ThreadPool, f: F) -> Self
    where
        A: Send,
        E: Send,
        F: FnOnce() -> Result<A, E> + Send + 'static,
    {
        let (send, recv) = channel();

        pool.spawn(move || {
            let res = f();
            let _ = send.send(res);
        });

        SpawnedFuture(recv)
    }
}

impl<A, E> Future for SpawnedFuture<A, E> {
    type Item = A;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.0.poll().expect("Sender destroyed") {
            Async::Ready(x) => x.map(Async::Ready),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
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

/// An `Allocator`, holding a counter for producing unique IDs for the stores.
#[derive(Debug, Default)]
struct Allocator {
    store_count: AtomicUsize,
}

impl Allocator {
    /// Creates a new `Allocator`.
    fn new() -> Self {
        Default::default()
    }

    /// Produces a new store id.
    fn next_store_id(&self) -> StoreId {
        StoreId(self.store_count.fetch_add(1, Ordering::Relaxed))
    }
}

struct StoreWithId<S: Store = Box<AnyStore>> {
    id: StoreId,
    store: S,
}

impl<S> StoreWithId<S>
where
    S: Store,
{
    fn id(&self) -> StoreId {
        self.id
    }
    fn store(&self) -> &S {
        &self.store
    }
}

/// The asset loader, holding the contexts,
/// the default (directory) store and a reference to the
/// `ThreadPool`.
pub struct Loader {
    contexts: FnvHashMap<TypeId, Box<Any + Send + Sync>>,
    directory: StoreWithId<Directory>,
    pool: Arc<ThreadPool>,
    stores: FnvHashMap<String, StoreWithId>,
    allocator: Allocator,
}

impl Loader {
    /// Creates a new asset loader, initializing the directory store with the
    /// given path.
    pub fn new<P>(directory: P, pool: Arc<ThreadPool>) -> Self
    where
        P: Into<PathBuf>,
    {
        let allocator = Allocator::new();
        Loader {
            contexts: Default::default(),
            directory: StoreWithId {
                id: allocator.next_store_id(),
                store: Directory::new(directory),
            },
            pool: pool,
            stores: Default::default(),
            allocator: allocator,
        }
    }

    /// Adds a store which can later be loaded from by supplying the same `name`
    /// to `load_from`.
    pub fn add_store<I, S>(&mut self, name: I, store: S)
    where
        I: Into<String>,
        S: Store + Send + Sync + 'static,
    {
        let id = self.allocator.next_store_id();
        self.stores.insert(
            name.into(),
            StoreWithId {
                id: id,
                store: Box::new(store) as Box<AnyStore>,
            },
        );
    }

    /// Registers an asset and inserts a context into the map.
    pub fn register<A, C>(&mut self, context: C)
    where
        A: Asset + 'static,
        C: Context<Asset = A>,
    {
        self.contexts
            .insert(TypeId::of::<A>(), Box::new(Arc::new(context)));
    }

    /// Like `load_from`, but doesn't ask the cache for the asset.
    pub fn reload<A, F, N, S>(&self, name: N, format: F, store: &S) -> AssetFuture<A>
    where
        A: Asset,
        F: Format + 'static,
        F::Data: Into<<A::Context as Context>::Data>,
        F::Error: 'static,
        N: Into<String>,
        S: Eq + Hash + ?Sized,
        String: Borrow<S>,
    {
        let context = self.context::<A::Context>();

        let si = self.store(store);

        reload_asset::<A, F, N, _>(
            context.clone(),
            format,
            name,
            si.id(),
            si.store(),
            &self.pool,
        )
    }

    /// Loads an asset with a given format from the default (directory) store.
    /// If you want to load from a custom source instead, use `load_from`.
    ///
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    pub fn load<A, F, N>(&self, id: N, format: F) -> AssetFuture<A>
    where
        A: Asset,
        F: Format + 'static,
        F::Data: Into<<A::Context as Context>::Data>,
        N: Into<String>,
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
    pub fn load_from<A, F, N, S>(&self, name: N, format: F, store: &S) -> AssetFuture<A>
    where
        A: Asset,
        F: Format + 'static,
        F::Data: Into<<A::Context as Context>::Data>,
        N: Into<String>,
        S: AsRef<str> + Eq + Hash + ?Sized,
        String: Borrow<S>,
    {
        let context = self.context::<A::Context>();
        let (ref store, id) = match store.as_ref() {
            "" => {
                let si = &self.directory;
                (si.store() as &AnyStore, si.id())
            }
            _ => {
                let si = self.store(store);
                (si.store() as &AnyStore, si.id())
            }
        };

        load_asset::<A, F, N, _>(context.clone(), format, name, id, store, &self.pool)
    }

    /// Loads an asset with a given id and format from a custom store.
    /// The actual work is done on a worker thread, thus this method immediately returns
    /// a future.
    ///
    /// # Panics
    ///
    /// Panics if the asset wasn't registered.
    pub fn load_data<A>(&self, data: <A::Context as Context>::Data) -> AssetFuture<A>
    where
        A: Asset,
    {
        AssetFuture::from_future(
            self.context::<A::Context>()
                .create_asset(data, &self.pool)
                .into_future()
                .map_err(BoxedErr::new),
        )
    }

    fn context<C>(&self) -> &Arc<C>
    where
        C: Context,
    {
        let context = self.contexts
            .get(&TypeId::of::<C::Asset>())
            .expect("Assets need to be registered with `Loader::register`.");

        // `Any + Send + Sync` doesn't have `downcast_ref`
        Any::downcast_ref(&**context).unwrap()
    }

    fn store<S>(&self, store: &S) -> &StoreWithId
    where
        S: Eq + Hash + ?Sized,
        String: Borrow<S>,
    {
        self.stores
            .get(&store)
            .expect("No such store. Maybe you forgot to add it with `Loader::add_store`?")
    }
}

/// Loads an asset with a given context, format, specifier and storage right now.
pub fn load_asset<A, F, N, S>(
    context: Arc<A::Context>,
    format: F,
    name: N,
    store_id: StoreId,
    storage: &S,
    pool: &Arc<ThreadPool>,
) -> AssetFuture<A>
where
    A: Asset,
    A::Context: Context,
    F: Format + 'static,
    F::Data: Into<<A::Context as Context>::Data>,
    F::Error: 'static,
    N: Into<String>,
    S: Store + ?Sized,
    <S::Result as IntoFuture>::Future: 'static,
{
    let name = name.into();

    let spec = AssetSpec::new(name.clone(), F::EXTENSIONS, store_id);

    context.retrieve(&spec).unwrap_or_else(move || {
        load_asset_inner(context, format, spec, storage, pool)
    })
}

/// Loads an asset with a given context, format, specifier and storage right now.
/// Note that this method does not ask for a cached version of the asset, but just
/// reloads the asset.
pub fn reload_asset<A, F, N, S>(
    context: Arc<A::Context>,
    format: F,
    name: N,
    store_id: StoreId,
    storage: &S,
    pool: &Arc<ThreadPool>,
) -> AssetFuture<A>
where
    A: Asset,
    A::Context: Context,
    F: Format + 'static,
    F::Data: Into<<A::Context as Context>::Data>,
    F::Error: 'static,
    N: Into<String>,
    S: Store + ?Sized,
    <S::Result as IntoFuture>::Future: 'static,
{
    let name = name.into();

    let spec = AssetSpec::new(name.clone(), F::EXTENSIONS, store_id);

    load_asset_inner(context, format, spec, storage, pool)
}

fn load_asset_inner<C, F, S>(
    context: Arc<C>,
    format: F,
    spec: AssetSpec,
    store: &S,
    pool: &Arc<ThreadPool>,
) -> AssetFuture<C::Asset>
where
    C: Context,
    F: Format + 'static,
    F::Data: Into<C::Data>,
    F::Error: 'static,
    S: Store + ?Sized,
    <S::Result as IntoFuture>::Future: 'static,
{
    let spec_store_err = spec.clone();
    let spec_format_err = spec.clone();
    let spec_asset_err = spec.clone();
    let context_clone = context.clone();
    let pool = pool.clone();
    let pool_clone = pool.clone();
    let future = store
        .load(context.category(), &spec.name, spec.exts)
        .into_future()
        .map_err(LoadError::StorageError::<C::Error, F::Error, S::Error>)
        .map_err(move |e| AssetError::new(spec_store_err, e))
        .and_then(move |bytes| {
            format
                .parse(bytes, &pool)
                .into_future()
                .map(Into::into)
                .map_err(LoadError::FormatError::<C::Error, F::Error, S::Error>)
                .map_err(move |e| AssetError::new(spec_format_err, e))
        })
        .and_then(move |data| {
            context
                .create_asset(data, &pool_clone)
                .into_future()
                .map_err(LoadError::AssetError::<C::Error, F::Error, S::Error>)
                .map_err(move |e| AssetError::new(spec_asset_err, e))
        })
        .map_err(BoxedErr::new);

    let future: Box<Future<Item = C::Asset, Error = BoxedErr>> = Box::new(future);
    let future = AssetFuture::from(future.shared());

    context_clone.cache(spec, future.clone());

    future
}
