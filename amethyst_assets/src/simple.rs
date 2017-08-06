//! Offers simple implementations which are applicable for
//! many use-cases.

use std::borrow::Cow;
use std::error::Error;
use std::marker::PhantomData;

use {Asset, AssetSpec, Cache, Context};

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

    fn from_data(&self, data: Self::Data) -> Result<Self::Asset, Self::Error> {
        (&self.load)(data)
    }

    fn cache(&self, spec: AssetSpec, asset: &Self::Asset) {
        self.cache.insert(spec, asset.clone());
    }

    fn retrieve(&self, spec: &AssetSpec) -> Option<Self::Asset> {
        self.cache.get(spec)
    }

    fn clear(&self) {
        self.cache.retain(|_, a| a.is_shared());
    }

    fn clear_all(&self) {
        self.cache.clear_all();
    }
}
