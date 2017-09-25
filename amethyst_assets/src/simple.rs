//! Offers simple implementations which are applicable for
//! many use-cases.

use std::borrow::Cow;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{Async, IntoFuture};
use futures::Future;
use parking_lot::RwLock;
use rayon::ThreadPool;

use {Asset, AssetFuture, AssetSpec, Cache, Context};

struct AssetUpdate<A, W> {
    counter: AtomicUsize,
    ready: RwLock<Option<A>>,
    defer: RwLock<Option<AssetFuture<W>>>,
}

impl<A, W> AssetUpdate<A, W> {
    fn new() -> Self {
        AssetUpdate {
            counter: AtomicUsize::new(0),
            ready: RwLock::new(None),
            defer: RwLock::new(None),
        }
    }
}

impl<A, W> AssetUpdate<A, W>
where
    A: Clone,
    W: AsRef<A> + Clone,
{
    fn push_update(&self, mut updated: AssetFuture<W>) {
        match updated.poll() {
            Ok(Async::Ready(updated)) => {
                *self.ready.write() = Some(updated.as_ref().clone());
                self.counter.fetch_add(1, Ordering::Release);
            }
            Err(_) => {}
            Ok(Async::NotReady) => {
                let last = {
                    let defer_lock = &mut *self.defer.write();
                    let last = defer_lock.take();
                    *defer_lock = Some(updated);
                    last
                };
                if let Some(mut last) = last {
                    match last.poll() {
                        Err(_) | Ok(Async::NotReady) => {}
                        Ok(Async::Ready(updated)) => {
                            *self.ready.write() = Some(updated.as_ref().clone());
                        }
                    }
                }
                self.counter.fetch_add(1, Ordering::Release);
            }
        }
    }

    fn updated(&self, count: usize) -> Option<(A, usize)> {
        let new = self.counter.load(Ordering::Acquire);
        if new > count {
            match self.defer
                .read()
                .as_ref()
                .map(AssetFuture::peek)
                .and_then(|a| a)
            {
                Some(Ok(updated)) => Some((updated.as_ref().clone(), new)),
                Some(Err(_)) | None => self.ready.read().as_ref().map(|a| (a.clone(), new)),
            }
        } else {
            None
        }
    }
}

/// An `AssetPtr` which provides `push_update`, `update`
/// and `is_shared` methods. These can simply be called
/// in order to implement the `Asset` trait.
///
/// The recommended strategy is to create a struct for an
/// asset which simply wraps `AssetPtr` and implements
/// `Asset` by calling these methods. Methods for the asset
/// can then be implemented on that wrapper struct by getting
/// the inner asset with `inner` and `inner_mut`.
///
/// The type parameter `A` is the type of the asset handle
/// (examples: texture handle, shader id, ..). To avoid unnecessarily
/// duplicated buffer allocations, make sure your handle is reference-counted,
/// so wrap it with an `Arc` in case the handle doesn't have this functionality
/// itself.
#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct AssetPtr<A, W> {
    inner: A,
    version: usize,
    #[derivative(Debug = "ignore")]
    update: Arc<AssetUpdate<A, W>>,
}

impl<A, W> AssetPtr<A, W> {
    /// Creates a new asset pointer.
    pub fn new(data: A) -> Self {
        AssetPtr {
            inner: data,
            update: Arc::new(AssetUpdate::new()),
            version: 0,
        }
    }

    /// Take the inner asset.
    pub fn inner(self) -> A {
        self.inner
    }

    /// Borrows the inner asset.
    pub fn inner_ref(&self) -> &A {
        &self.inner
    }

    /// Borrows the inner asset mutably.
    pub fn inner_mut(&mut self) -> &mut A {
        &mut self.inner
    }

    /// Returns `true` if a clone of this `AssetPtr` exists.
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.update) > 1
    }
}

impl<A, W> AssetPtr<A, W>
where
    A: Clone,
    W: AsRef<A> + Clone,
{
    /// Pushes an update to the shared update container;
    /// this update can then be applied to all asset pointers by calling
    /// `update` on them.
    pub fn push_update(&self, updated: AssetFuture<W>) {
        self.update.push_update(updated);
    }

    /// Applies a previously pushed update.
    pub fn update(&mut self) {
        if let Some((updated, version)) = self.update.updated(self.version) {
            self.inner = updated;
            self.version = version;
        }
    }
}

/// `Asset` implementation that supports hot reloading
pub struct SimpleAsset<A>(pub AssetPtr<A, SimpleAsset<A>>);
impl<A> AsRef<A> for SimpleAsset<A> {
    fn as_ref(&self) -> &A {
        self.0.inner_ref()
    }
}
impl<A> AsMut<A> for SimpleAsset<A> {
    fn as_mut(&mut self) -> &mut A {
        self.0.inner_mut()
    }
}

/// A simple implementation of the `Context` trait.
pub struct SimpleContext<A, D, T> {
    cache: Cache<AssetFuture<SimpleAsset<A>>>,
    category: Cow<'static, str>,
    load: T,
    phantom: PhantomData<*const D>,
}

impl<A, D, T> SimpleContext<A, D, T> {
    /// Creates a new `SimpleContext` from a category string and
    /// a closure which transforms data to assets.
    pub fn new<C: Into<Cow<'static, str>>>(category: C, load: T) -> Self {
        SimpleContext {
            cache: Cache::new(),
            category: category.into(),
            load,
            phantom: PhantomData,
        }
    }
}

unsafe impl<A, D, T> Send for SimpleContext<A, D, T>
where
    T: Send,
{
}

unsafe impl<A, D, T> Sync for SimpleContext<A, D, T>
where
    T: Sync,
{
}

impl<A, D, E, R, T> Context for SimpleContext<A, D, T>
where
    A: Clone,
    R: Send + 'static,
    D: Send + 'static,
    T: Fn(D) -> R + Send + Sync + 'static,
    SimpleAsset<A>: Asset + Clone + Send + 'static,
    E: Error + Send + Sync,
    R: IntoFuture<Item = SimpleAsset<A>, Error = E>,
{
    type Asset = SimpleAsset<A>;
    type Data = D;
    type Error = E;
    type Result = R;

    fn category(&self) -> &str {
        self.category.as_ref()
    }

    fn create_asset(&self, data: Self::Data, _: &ThreadPool) -> R {
        (&self.load)(data)
    }

    fn cache(&self, spec: AssetSpec, asset: AssetFuture<SimpleAsset<A>>) {
        self.cache.insert(spec, asset);
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<AssetFuture<SimpleAsset<A>>> {
        self.cache.get(spec)
    }

    fn update(&self, spec: &AssetSpec, updated: AssetFuture<SimpleAsset<A>>) {
        if let Some(updated) = self.cache
            .access(spec, |a| match a.peek() {
                Some(Ok(a)) => {
                    a.0.push_update(updated);
                    None
                }
                _ => Some(updated),
            })
            .and_then(|a| a)
        {
            self.cache.insert(spec.clone(), updated);
        }
    }

    fn clear(&self) {
        self.cache.retain(|_, a| match a.peek() {
            Some(Ok(a)) => a.0.is_shared(),
            _ => true,
        });
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}
