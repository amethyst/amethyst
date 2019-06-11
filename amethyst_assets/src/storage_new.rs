use atelier_loader::{slotmap, LoadHandle};
use crossbeam::queue::SegQueue;

use crate::{
    asset::{Asset},
    loader_new::AssetHandle,
};

struct AssetState<A> {
    version: u32,
    asset: A,
}
/// An asset storage, storing the actual assets and allocating
/// handles to them.
pub struct AssetStorage<A: Asset> {
    assets: slotmap::SecondaryMap<LoadHandle, AssetState<A>>,
    uncommitted: slotmap::SecondaryMap<LoadHandle, AssetState<A>>,
    to_drop: SegQueue<A>,
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn update_asset(&mut self, handle: &LoadHandle, asset: A, version: u32) {
        if let Some(data) = self.uncommitted.remove(*handle) {
            // uncommitted data already exists for the handle, drop it
            self.to_drop.push(data.asset);
        }
        self.uncommitted
            .insert(*handle, AssetState { version, asset });
    }

    pub(crate) fn remove_asset(&mut self, handle: LoadHandle) {
        if let Some(data) = self.uncommitted.remove(handle) {
            self.to_drop.push(data.asset);
        }
        if let Some(data) = self.assets.remove(handle) {
            self.to_drop.push(data.asset);
        }
    }

    pub(crate) fn commit_asset(&mut self, handle: &LoadHandle, version: u32) {
        if let Some(data) = self.uncommitted.remove(*handle) {
            if data.version != version {
                panic!("attempted to commit asset version which mismatches with existing uncommitted version")
            }
            if let Some(existing) = self.assets.remove(*handle) {
                // data already exists for the handle, drop it
                self.to_drop.push(existing.asset);
            }
            self.assets.insert(
                *handle,
                AssetState {
                    version,
                    asset: data.asset,
                },
            );
        } else {
            panic!("attempted to commit asset which doesn't exist");
        }
    }

    // TODO implement this as a pub(crate) function for usage by Loader and let Loader manage handle allocation

    // /// When cloning an asset handle, you'll get another handle,
    // /// but pointing to the same asset. If you instead want to
    // /// indeed create a new asset, you can use this method.
    // /// Note however, that it needs a mutable borrow of `self`,
    // /// so it can't be used in parallel.
    // pub fn clone_asset(&mut self, handle: &Handle<A>) -> Option<Handle<A>>
    // where
    //     A: Clone,

    /// Get an asset from a given asset handle.
    pub fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        self.assets.get(*handle.get_load_handle()).map(|a| &a.asset)
    }

    /// Get an asset mutably from a given asset handle.
    pub fn get_mut<T: AssetHandle>(&mut self, handle: &T) -> Option<&mut A> {
        self.assets
            .get_mut(*handle.get_load_handle())
            .map(|a| &mut a.asset)
    }

    pub fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        self.assets
            .get(*handle.get_load_handle())
            .map(|a| a.version)
    }

    pub fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        self.assets
            .get(*handle.get_load_handle())
            .map(|a| (&a.asset, a.version))
    }

    /// Process finished asset data and maintain the storage.
    /// This calls the `drop_fn` closure for assets that were removed from the storage.
    pub fn process_custom_drop<D>(&mut self, mut drop_fn: D)
    where
        D: FnMut(A),
    {
        while let Some(asset) = self.to_drop.try_pop() {
            drop_fn(asset);
        }
    }
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            assets: Default::default(),
            uncommitted: Default::default(),
            to_drop: SegQueue::new(),
        }
    }
}
