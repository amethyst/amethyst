use std::collections::HashMap;

use atelier_assets::loader::{handle::AssetHandle, storage::IndirectionTable, LoadHandle};
use crossbeam_queue::SegQueue;
use log::debug;

struct AssetState<A> {
    version: u32,
    asset: A,
}

/// An asset storage, storing the actual assets
///
/// # Type Parameters
///
/// * `A`: Asset Rust type.
pub struct AssetStorage<A> {
    assets: HashMap<LoadHandle, AssetState<A>>,
    uncommitted: HashMap<LoadHandle, AssetState<A>>,
    to_drop: SegQueue<A>,
    indirection_table: IndirectionTable,
}

impl<A> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new(indirection_table: IndirectionTable) -> Self {
        Self {
            assets: Default::default(),
            uncommitted: Default::default(),
            to_drop: SegQueue::new(),
            indirection_table,
        }
    }

    /// Added to make api compatible with previous storage
    pub fn unload_all(&mut self) {
        // FIXME do the unload correctly
        self.assets.clear();
    }

    pub(crate) fn update_asset(&mut self, handle: LoadHandle, asset: A, version: u32) {
        debug!(
            "AssetStorage<A>::update_asset load_handle: {:?}, version: {}",
            handle, version
        );
        if let Some(data) = self.uncommitted.remove(&handle) {
            debug!(
                "uncommitted data already exists for the handle: {:?}, drop it",
                handle
            );
            // uncommitted data already exists for the handle, drop it
            self.to_drop.push(data.asset);
        }
        debug!(
            "insert asset with load_handle: {:?}, version: {}",
            handle, version
        );
        self.uncommitted
            .insert(handle, AssetState { version, asset });
    }

    pub(crate) fn remove_asset(&mut self, handle: LoadHandle, version: u32) {
        if let Some(data) = self.uncommitted.get(&handle) {
            if data.version == version {
                self.to_drop
                    .push(self.uncommitted.remove(&handle).unwrap().asset);
            }
        }
        if let Some(data) = self.assets.get(&handle) {
            if data.version == version {
                self.to_drop
                    .push(self.uncommitted.remove(&handle).unwrap().asset);
            }
        }
    }

    pub(crate) fn commit_asset(&mut self, handle: LoadHandle, version: u32) {
        if let Some(data) = self.uncommitted.remove(&handle) {
            if data.version != version {
                panic!("attempted to commit asset version which mismatches with existing uncommitted version")
            }
            if let Some(existing) = self.assets.remove(&handle) {
                // data already exists for the handle, drop it
                self.to_drop.push(existing.asset);
            }
            self.assets.insert(
                handle,
                AssetState {
                    version,
                    asset: data.asset,
                },
            );
        } else {
            panic!("attempted to commit asset which doesn't exist");
        }
    }

    /// returns true when asset is loaded for this handle
    pub fn contains(&self, load_handle: LoadHandle) -> bool {
        let load_handle = if load_handle.is_indirect() {
            if let Some(handle) = self.indirection_table.resolve(load_handle) {
                handle
            } else {
                return false;
            }
        } else {
            load_handle
        };
        self.assets.contains_key(&load_handle)
    }

    fn get_asset_state(&self, load_handle: LoadHandle) -> Option<&AssetState<A>> {
        let load_handle = if load_handle.is_indirect() {
            self.indirection_table.resolve(load_handle)?
        } else {
            load_handle
        };
        self.assets.get(&load_handle)
    }

    /// Returns the asset for the given load handle, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `load_handle`: LoadHandle of the asset.
    pub fn get_for_load_handle(&self, load_handle: LoadHandle) -> Option<&A> {
        self.get_asset_state(load_handle).map(|a| &a.asset)
    }

    /// Returns the asset for the given handle, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset handle type.
    pub fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        self.get_asset_state(handle.load_handle()).map(|a| &a.asset)
    }

    /// Returns the version of a loaded asset, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset handle type.
    pub fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        self.get_asset_state(handle.load_handle())
            .map(|a| a.version)
    }

    /// Returns the loaded asset and its version, or `None` if has not completed loading.
    ///
    /// # Parameters
    ///
    /// * `handle`: Handle of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset handle type.
    pub fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        self.get_asset_state(handle.load_handle())
            .map(|a| (&a.asset, a.version))
    }

    /// Process finished asset data and maintain the storage.
    ///
    /// This calls the `drop_fn` function for assets that were removed from the storage.
    ///
    /// # Parameters
    ///
    /// * `drop_fn`: The function to invoke with the asset.
    ///
    /// # Type Parameters
    ///
    /// * `D`: Drop function type.
    pub fn process_custom_drop<D>(&mut self, mut drop_fn: D)
    where
        D: FnMut(A),
    {
        while let Ok(asset) = self.to_drop.pop() {
            drop_fn(asset);
        }
    }
}

impl<A> atelier_assets::loader::handle::TypedAssetStorage<A> for AssetStorage<A> {
    fn get<T: AssetHandle>(&self, handle: &T) -> Option<&A> {
        self.get(handle)
    }
    fn get_version<T: AssetHandle>(&self, handle: &T) -> Option<u32> {
        self.get_version(handle)
    }
    fn get_asset_with_version<T: AssetHandle>(&self, handle: &T) -> Option<(&A, u32)> {
        self.get_asset_with_version(handle)
    }
}
