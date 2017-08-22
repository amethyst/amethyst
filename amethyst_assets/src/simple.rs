//! Offers simple implementations which are applicable for
//! many use-cases.

use futures::Future;

use std::borrow::Cow;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{Async, IntoFuture};
use futures::future::Shared;
use parking_lot::RwLock;
use rayon::ThreadPool;

use {Asset, AssetFuture, AssetSpec, Cache, Context};

struct AssetUpdate<A> {
    counter: AtomicUsize,
    ready: RwLock<Option<A>>,
    defer: RwLock<Option<AssetFuture<AssetPtr<A>>>>,
}

impl<A> AssetUpdate<A>
    where A: Clone
{
    fn new() -> Self {
        AssetUpdate {
            counter: AtomicUsize::new(0),
            ready: RwLock::new(None),
            defer: RwLock::new(None),
        }
    }

    fn push_update(&self, mut updated: AssetFuture<AssetPtr<A>>) {
        match updated.poll() {
            Ok(Async::Ready(updated)) => {
                *self.ready.write() = Some(updated.inner.clone());
                self.counter.fetch_add(1, Ordering::Release);
            }
            Err(_) => {},
            Ok(Async::NotReady) => {
                let last = {
                    let defer_lock = &mut*self.defer.write();
                    let last = defer_lock.take();
                    *defer_lock = Some(updated);
                    last
                };
                if let Some(mut last) = last {
                    match last.poll() {
                        Err(_) | Ok(Async::NotReady) => {}
                        Ok(Async::Ready(updated)) => {
                            *self.ready.write() = Some(updated.inner.clone());
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
            match self.defer.read().as_ref().map(Shared::peek).and_then(|a|a) {
                Some(Ok(updated)) => {
                    Some((updated.inner.clone(), new))
                }
                Some(Err(_)) | None => {
                    self.ready.read().as_ref().map(|a| (a.clone(), new))
                }
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
#[derive(Clone)]
pub struct AssetPtr<A> {
    inner: A,
    version: usize,
    update: Arc<AssetUpdate<A>>,
}

impl<A> AssetPtr<A>
    where A: Clone,
{
    /// Creates a new asset pointer.
    pub fn new(data: A) -> Self {
        AssetPtr {
            inner: data,
            update: Arc::new(AssetUpdate::new()),
            version: 0,
        }
    }

    /// Borrows the inner asset.
    pub fn inner(&self) -> &A {
        &self.inner
    }

    /// Borrows the inner asset mutably.
    pub fn inner_mut(&mut self) -> &mut A {
        &mut self.inner
    }

    /// Pushes an update to the shared update container;
    /// this update can then be applied to all asset pointers by calling
    /// `update` on them.
    pub fn push_update(&self, updated: AssetFuture<AssetPtr<A>>) {
        self.update.push_update(updated);
    }

    /// Applies a previously pushed update.
    pub fn update(&mut self) {
        if let Some((updated, version)) = self.update.updated(self.version) {
            self.inner = updated;
            self.version = version;
        }
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.update) > 1
    }
}

/// A simple implementation of the `Context` trait.
pub struct SimpleContext<A, D, R, T> {
    cache: Cache<AssetFuture<AssetPtr<A>>>,
    category: Cow<'static, str>,
    load: T,
    phantom: PhantomData<(D, R)>,
}

impl<A, D, E, T> SimpleContext<A, D, E, T> {
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

impl<A, D, E, R, T> Context for SimpleContext<A, D, R, T>
    where A: Clone,
          R: Send + 'static,
          D: Send + 'static,
          T: Fn(D) -> R + Send + 'static,
          AssetPtr<A>: Asset + Clone + Send + 'static,
          E: Error + Send + Sync,
          R: IntoFuture<Item=AssetPtr<A>, Error=E>,
{
    type Asset = AssetPtr<A>;
    type Data = D;
    type Error = E;
    type Result = R;

    fn category(&self) -> &str {
        self.category.as_ref()
    }

    fn create_asset(&self, data: Self::Data, pool: &ThreadPool) -> R {
        (&self.load)(data)
    }

    fn cache(&self, spec: AssetSpec, asset: AssetFuture<AssetPtr<A>>) {
        self.cache.insert(spec, asset);
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<AssetFuture<AssetPtr<A>>> {
        self.cache.get(spec)
    }

    fn update(&self, spec: &AssetSpec, updated: AssetFuture<AssetPtr<A>>) {
        if let Some(updated) = self.cache.access(spec, |a| {
            match a.peek() {
                Some(Ok(a)) => { a.push_update(updated); None }
                _ => { Some(updated) }
            }
        }).and_then(|a|a) {
            self.cache.insert(spec.clone(), updated);
        }
    }

    fn clear(&self) {
        self.cache.retain(|_, a| match a.peek() {
            Some(Ok(a)) => { a.is_shared() }
            _ => { true }
        });
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}
