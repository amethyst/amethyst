use std::{borrow::Borrow, hash::Hash};

use derivative::Derivative;
use fnv::FnvHashMap;

use crate::{Handle, WeakHandle};

/// A simple cache for asset handles of type `A`.
/// This stores `WeakHandle`, so it doesn't keep the assets alive.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Cache<A> {
    map: FnvHashMap<String, WeakHandle<A>>,
}

impl<A> Cache<A>
where
    A: Clone,
{
    /// Creates a new `Cache` and initializes it with the default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Inserts an asset with a given `key` and returns the old value (if any).
    pub fn insert<K: Into<String>>(&mut self, key: K, asset: &Handle<A>) -> Option<WeakHandle<A>> {
        self.map.insert(key.into(), asset.downgrade())
    }

    /// Retrieves an asset handle using a given `key`.
    pub fn get<K>(&self, key: &K) -> Option<Handle<A>>
    where
        K: ?Sized + Hash + Eq,
        String: Borrow<K>,
    {
        self.map.get(key).and_then(WeakHandle::upgrade)
    }

    /// Deletes all cached handles which are invalid.
    pub fn clear_dead<F>(&mut self) {
        self.map.retain(|_, h| !h.is_dead());
    }

    /// Clears all values.
    pub fn clear_all(&mut self) {
        self.map.clear();
    }
}
