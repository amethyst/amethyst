//! Offers simple implementations which are applicable for
//! many use-cases.

use std::borrow::Cow;
use std::error::Error;
use std::marker::PhantomData;

use futures::IntoFuture;
use rayon::ThreadPool;

use {Asset, AssetUpdates, AssetSpec, Cache, Context};

/// A simple implementation of the `Context` trait.
pub struct SimpleContext<A, D, T> {
    cache: Cache<AssetUpdates<A>>,
    category: Cow<'static, str>,
    load: T,
    phantom: PhantomData<fn() -> D>,
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
    A: Asset + Clone + Send + 'static,
    E: Error + Clone + Send + Sync,
    R: IntoFuture<Item = A, Error = E>,
{
    type Asset = A;
    type Data = D;
    type Error = E;
    type Result = R;

    fn category(&self) -> &str {
        self.category.as_ref()
    }

    fn create_asset(&self, data: Self::Data, _: &ThreadPool) -> R {
        (&self.load)(data)
    }

    fn cache(&self, spec: AssetSpec, asset: AssetUpdates<A>) {
        self.cache.insert(spec, asset);
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<AssetUpdates<A>> {
        self.cache.get(spec)
    }

    fn clear(&self) {
        self.cache.retain(|_, a| a.is_shared())
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}
