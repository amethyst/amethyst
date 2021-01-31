use std::{borrow::Borrow, hash::Hash, marker::PhantomData};

use distill::loader::{
    crossbeam_channel::Sender,
    handle::{AssetHandle, Handle, RefOp, WeakHandle},
};
use fnv::FnvHashMap;

/// A simple cache for asset handles of type `A`.
/// This stores `WeakHandle`, so it doesn't keep the assets alive.
// #[derive(Derivative)]
// #[derivative(Default(bound = ""))]
pub struct Cache<A> {
    map: FnvHashMap<String, WeakHandle>,
    tx: Sender<RefOp>,
    marker: PhantomData<A>,
}

impl<A> Cache<A>
where
    A: Clone,
{
    /// Creates a new `Cache` and initializes it with the default values.
    pub fn new(tx: Sender<RefOp>) -> Self {
        Self {
            map: Default::default(),
            tx,
            marker: Default::default(),
        }
    }

    /// Inserts an asset with a given `key` and returns the old value (if any).
    pub fn insert<K: Into<String>>(&mut self, key: K, asset: &Handle<A>) -> Option<WeakHandle> {
        self.map.insert(key.into(), asset.downgrade())
    }

    /// Retrieves an asset handle using a given `key`.
    pub fn get<K>(&self, key: &K) -> Option<Handle<A>>
    where
        K: ?Sized + Hash + Eq,
        String: Borrow<K>,
    {
        self.map.get(key).map(|weak_handle: &WeakHandle| {
            Handle::<A>::new(self.tx.clone(), weak_handle.load_handle())
        })
    }

    /// Clears all values.
    pub fn clear_all(&mut self) {
        self.map.clear();
    }
}
