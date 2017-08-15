//! Offers simple implementations which are applicable for
//! many use-cases.

use std::borrow::Cow;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::RwLock;

use {Asset, AssetSpec, Cache, Context};

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
    update: Arc<(AtomicUsize, RwLock<Option<A>>)>,
    version: usize,
}

impl<A> AssetPtr<A> {
    /// Creates a new asset pointer.
    pub fn new(data: A) -> Self {
        AssetPtr {
            inner: data,
            update: Arc::new((AtomicUsize::new(0), RwLock::new(None))),
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
}

impl<A> AssetPtr<A>
    where A: Clone
{
    /// Pushes an update to the shared update container;
    /// this update can then be applied to all asset pointers by calling
    /// `update` on them.
    pub fn push_update(&self, updated: Self) {
        let &(ref count, ref lock) = &*self.update;

        *lock.write() = Some(updated.inner);
        count.fetch_add(1, Ordering::Release);
    }

    /// Applies a previously pushed update.
    pub fn update(&mut self) {
        let &(ref count, ref lock) = &*self.update;

        let new_count = count.load(Ordering::Acquire);
        if new_count != self.version {
            self.inner = lock.read().as_ref().expect("Unexpected None").clone();
            self.version = new_count;
        }
    }

    /// Returns `true` if a clone of this `AssetPtr` exists.
    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.update) > 1
    }
}

/// A simple implementation of the `Context` trait.
pub struct SimpleContext<A, D, E, T> {
    cache: Cache<A>,
    category: Cow<'static, str>,
    load: T,
    phantom: PhantomData<(D, E)>,
}

impl<A, D, E, T> SimpleContext<A, D, E, T>
    where A: Asset,
{
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

impl<A, D, E, T> Context for SimpleContext<A, D, E, T>
    where T: Fn(D) -> Result<A, E>,
          A: Asset,
          E: Error,
{
    type Asset = A;
    type Data = D;
    type Error = E;

    fn category(&self) -> &str {
        self.category.as_ref()
    }

    fn create_asset(&self, data: Self::Data) -> Result<Self::Asset, Self::Error> {
        (&self.load)(data)
    }

    fn cache(&self, spec: AssetSpec, asset: &Self::Asset) {
        self.cache.insert(spec, asset.clone());
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<Self::Asset> {
        self.cache.get(spec)
    }

    fn update(&self, spec: &AssetSpec, updated: A) {
        if !self.cache.access(spec, |a| a.push_update(updated)) {
            warn!(target: "SimpleContext::update",
                  "Cannot update the asset {:?} since there is no cached version",
                  spec);
        }
    }

    fn clear(&self) {
        self.cache.retain(|_, a| a.is_shared());
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}
