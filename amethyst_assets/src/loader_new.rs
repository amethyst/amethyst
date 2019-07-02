use crate::{processor::ProcessingQueue, storage_new::AssetStorage};
use amethyst_core::ecs::{
    prelude::{Component, DenseVecStorage},
    Resources,
};
use atelier_loader::{self, AssetLoadOp, AssetTypeId, Loader as AtelierLoader};
use bincode;
use crossbeam::channel::{unbounded, Receiver, Sender};
use derivative::Derivative;
use serde::de::Deserialize;
use std::{collections::HashMap, error::Error, marker::PhantomData, sync::Arc};

pub(crate) use atelier_loader::LoadHandle;
pub use atelier_loader::{AssetUuid, LoadStatus, TypeUuid};

/// Handle to an asset.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
pub struct Handle<T: ?Sized> {
    chan: Arc<Sender<RefOp>>,
    id: LoadHandle,
    marker: PhantomData<T>,
}

impl<T> Handle<T> {
    fn new(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self {
            chan,
            id: handle,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Drop for Handle<T> {
    fn drop(&mut self) {
        self.chan.send(RefOp::Decrease(self.id))
    }
}

impl<T> AssetHandle for Handle<T> {
    fn load_handle(&self) -> &LoadHandle {
        &self.id
    }
}

/// Handle to an asset whose type is unknown during loading.
///
/// This is returned by `Loader::load_asset_generic` for assets loaded by UUID.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
pub struct GenericHandle {
    chan: Arc<Sender<RefOp>>,
    id: LoadHandle,
}

impl GenericHandle {
    fn new(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self { chan, id: handle }
    }
}

impl Drop for GenericHandle {
    fn drop(&mut self) {
        self.chan.send(RefOp::Decrease(self.id))
    }
}

impl AssetHandle for GenericHandle {
    fn load_handle(&self) -> &LoadHandle {
        &self.id
    }
}

/// Handle to an asset that does not prevent the asset from being unloaded.
///
/// Weak handles are primarily used when you want to use something that is already loaded.
///
/// For example, a strong handle to an asset may be guaranteed to exist elsewhere in the program,
/// and so you can simply get and use a weak handle to that asset in other parts of your code. This
/// removes reference counting overhead, but also ensures that the system which uses the weak handle
/// is not in control of when to unload the asset.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
pub struct WeakHandle {
    id: LoadHandle,
}

impl WeakHandle {
    fn new(handle: LoadHandle) -> Self {
        WeakHandle { id: handle }
    }
}

impl AssetHandle for WeakHandle {
    fn load_handle(&self) -> &LoadHandle {
        &self.id
    }
}

impl<T: TypeUuid + Send + Sync + 'static> Component for Handle<T> {
    type Storage = DenseVecStorage<Handle<T>>;
}

/// The contract of an asset handle.
///
/// There are two types of asset handles:
///
/// * **Typed -- `Handle<T>`:** When the asset's type is known when loading.
/// * **Generic -- `GenericHandle`:** When only the asset's UUID is known when loading.
pub trait AssetHandle {
    /// Returns the load status of the asset.
    ///
    /// # Parameters
    ///
    /// * `loader`: Loader that is loading the asset.
    ///
    /// # Type Parameters
    ///
    /// * `L`: Asset loader type.
    fn load_status<L: Loader>(&self, loader: &L) -> LoadStatus {
        loader.get_load_status_handle(self.load_handle())
    }

    /// Returns an immutable reference to the asset if it is committed.
    ///
    /// # Parameters
    ///
    /// * `storage`: Asset storage.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn asset<'a, T: TypeUuid>(&self, storage: &'a AssetStorage<T>) -> Option<&'a T>
    where
        Self: Sized,
    {
        storage.get(self)
    }

    /// Returns a mutable reference to the asset if it is committed.
    ///
    /// # Parameters
    ///
    /// * `storage`: Asset storage.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn asset_mut<'a, T: TypeUuid>(&self, storage: &'a mut AssetStorage<T>) -> Option<&'a mut T>
    where
        Self: Sized,
    {
        storage.get_mut(self)
    }

    /// Returns the version of the asset if it is committed.
    ///
    /// # Parameters
    ///
    /// * `storage`: Asset storage.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn asset_version<'a, T: TypeUuid>(&self, storage: &'a AssetStorage<T>) -> Option<u32>
    where
        Self: Sized,
    {
        storage.get_version(self)
    }

    /// Returns the asset with the given version if it is committed.
    ///
    /// # Parameters
    ///
    /// * `storage`: Asset storage.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn asset_with_version<'a, T: TypeUuid>(
        &self,
        storage: &'a AssetStorage<T>,
    ) -> Option<(&'a T, u32)>
    where
        Self: Sized,
    {
        storage.get_asset_with_version(self)
    }

    /// Downgrades this handle into a `WeakHandle`.
    ///
    /// Be aware that if there are no longer any strong handles to the asset, then the underlying
    /// asset may be freed at any time.
    fn downgrade(&self) -> WeakHandle {
        WeakHandle::new(*self.load_handle())
    }

    /// Returns the `LoadHandle` of this asset handle.
    fn load_handle(&self) -> &LoadHandle;
}

/// Manages asset loading and storage for an application.
pub trait Loader: Send + Sync {
    /// Returns a generic asset handle and Loads the asset for the given UUID asynchronously.
    ///
    /// This is useful when loading an asset, but the asset's Rust type is unknown, such as for a
    /// loading screen that loads arbitrary assets.
    ///
    /// # Notes
    ///
    /// Be careful not to confuse `AssetUuid` with `AssetTypeUuid`:
    ///
    /// * `AssetUuid`: For an asset, such as "player_texture.png".
    /// * `AssetTypeUuid`: For an asset type, such as `Texture`.
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the asset.
    fn load_asset_generic(&self, id: AssetUuid) -> GenericHandle;

    /// Returns an asset handle and Loads the asset for the given UUID asynchronously.
    ///
    /// This is useful when loading an asset whose Rust type is known.
    ///
    /// # Notes
    ///
    /// Be careful not to confuse `AssetUuid` with `AssetTypeUuid`:
    ///
    /// * `AssetUuid`: For an asset, such as "player_texture.png".
    /// * `AssetTypeUuid`: For an asset type, such as `Texture`.
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the asset.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn load_asset<T: TypeUuid>(&self, id: AssetUuid) -> Handle<T>;

    /// Returns a weak handle to the asset of the given UUID, if any.
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the asset.
    fn get_load(&self, id: AssetUuid) -> Option<WeakHandle>;

    /// Returns the load status for the asset of the given UUID.
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the asset.
    fn get_load_status(&self, id: AssetUuid) -> LoadStatus {
        self.get_load(id)
            .map(|h| self.get_load_status_handle(h.load_handle()))
            .unwrap_or(LoadStatus::NotRequested)
    }

    /// Returns the load status for the asset with the given load handle.
    ///
    /// # Parameters
    ///
    /// * `handle`: `LoadHandle` of the asset.
    fn get_load_status_handle(&self, handle: &LoadHandle) -> LoadStatus;

    /// Returns an immutable reference to the asset if it is committed.
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the asset.
    /// * `storage`: Asset storage.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn get_asset<'a, T: TypeUuid>(
        &self,
        id: AssetUuid,
        storage: &'a AssetStorage<T>,
    ) -> Option<&'a T> {
        // TODO validate type for load
        self.get_load(id)
            .as_ref()
            .map(|h| storage.get(h))
            .unwrap_or(None)
    }

    /// Returns a mutable reference to the asset if it is committed.
    ///
    /// # Parameters
    ///
    /// * `id`: UUID of the asset.
    /// * `storage`: Asset storage.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn get_asset_mut<'a, T: TypeUuid>(
        &self,
        id: AssetUuid,
        storage: &'a mut AssetStorage<T>,
    ) -> Option<&'a mut T> {
        // TODO validate type for load
        if let Some(h) = self.get_load(id).as_ref() {
            storage.get_mut(h)
        } else {
            None
        }
    }

    /// Creates the `AssetTypeStorage`'s resources in the `World`.
    ///
    /// # Parameters
    ///
    /// * `resources`: Resources in the application.
    fn init_world(&mut self, resources: &mut Resources);

    /// Updates asset loading state and removes assets that are no longer referenced.
    ///
    /// # Parameters
    ///
    /// * `resources`: Resources in the application.
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>>;
}

/// Default loader is the Atelier Assets `RpcLoader`.
pub type DefaultLoader = LoaderWithStorage<atelier_loader::rpc_loader::RpcLoader<()>>;

/// Operations on an asset reference.
enum RefOp {
    Decrease(LoadHandle),
}

/// Asset loader and storage.
#[derive(Debug)]
pub struct LoaderWithStorage<T: AtelierLoader<HandleType = ()> + Send + Sync> {
    loader: T,
    storage_map: AssetStorageMap,
    ref_sender: Arc<Sender<RefOp>>,
    ref_receiver: Receiver<RefOp>,
}

impl<T: AtelierLoader<HandleType = ()> + Send + Sync + Default> Default for LoaderWithStorage<T> {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self {
            loader: Default::default(),
            storage_map: Default::default(),
            ref_sender: Arc::new(tx),
            ref_receiver: rx,
        }
    }
}

impl<T: AtelierLoader<HandleType = ()> + Send + Sync> Loader for LoaderWithStorage<T> {
    fn load_asset_generic(&self, id: AssetUuid) -> GenericHandle {
        GenericHandle::new(self.ref_sender.clone(), self.loader.add_ref(id))
    }
    fn load_asset<A: TypeUuid>(&self, id: AssetUuid) -> Handle<A> {
        Handle::new(self.ref_sender.clone(), self.loader.add_ref(id))
    }
    fn get_load(&self, id: AssetUuid) -> Option<WeakHandle> {
        self.loader.get_load(id).map(|h| WeakHandle::new(h))
    }
    fn get_load_status_handle(&self, handle: &LoadHandle) -> LoadStatus {
        self.loader.get_load_status(handle)
    }
    fn init_world(&mut self, resources: &mut Resources) {
        for (_, storage) in self.storage_map.storages_by_asset_uuid.iter() {
            (storage.create_storage)(resources);
        }
    }
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>> {
        loop {
            match self.ref_receiver.try_recv() {
                None => break,
                Some(RefOp::Decrease(ref handle)) => self.loader.remove_ref(handle),
            }
        }
        let storages = WorldStorages::new(resources, &self.storage_map);
        self.loader.process(&storages)
    }
}

/// Storage for a particular asset type.
///
/// This trait abtracts over the bridge between `atelier_loader` and Amethyst's asset storage. These
/// methods are called through dynamic dispatch by `atelier_loader` when an asset is loaded /
/// unloaded. All of these operations are performed on Amethyst's `AssetStorage`
pub trait AssetTypeStorage {
    /// Allocates and returns a handle for an asset.
    ///
    /// Currently the handle type is the empty tuple, i.e. no allocation is done.
    ///
    /// # Parameters
    ///
    /// * `handle`: The load handle of the asset.
    fn allocate(&self, handle: &LoadHandle) -> ();

    /// Updates an asset.
    ///
    /// # Parameters
    ///
    /// * `handle`: Load handle of the asset.
    /// * `data`: Asset data bytes (uncompressed).
    /// * `load_op`: Load operation to notify `atelier_loader` when the asset has been loaded.
    /// * `version`: Version of the asset -- this will be a new version for each hot reload.
    fn update_asset(
        &self,
        handle: &LoadHandle,
        data: &dyn AsRef<[u8]>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>>;

    /// Commits an asset.
    ///
    /// # Parameters
    ///
    /// * `handle`: Load handle of the asset.
    /// * `version`: Version of the asset -- this will be a new version for each hot reload.
    fn commit_asset_version(&mut self, handle: &LoadHandle, version: u32);

    /// Frees an asset.
    ///
    /// # Parameters
    ///
    /// * `handle`: Load handle of the asset.
    fn free(&mut self, handle: LoadHandle);
}

impl<Intermediate, Asset: TypeUuid + Send + Sync> AssetTypeStorage
    for (&ProcessingQueue<Intermediate>, &mut AssetStorage<Asset>)
where
    for<'a> Intermediate: Deserialize<'a> + TypeUuid + Send,
{
    fn allocate(&self, _load_handle: &LoadHandle) -> () {}
    fn update_asset(
        &self,
        handle: &LoadHandle,
        data: &dyn AsRef<[u8]>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>> {
        let asset = bincode::deserialize::<Intermediate>(data.as_ref())?;
        self.0.enqueue(*handle, asset, load_op, version);
        Ok(())
    }
    fn commit_asset_version(&mut self, handle: &LoadHandle, version: u32) {
        self.1.commit_asset(handle, version);
    }
    fn free(&mut self, handle: LoadHandle) {
        self.1.remove_asset(handle);
    }
}

/// Maps of `AssetType`, keyed by either the asset data UUID, or asset type UUID.
///
/// Used to discover the Rust `AssetType`, given the asset data UUID or the asset type UUID.
#[derive(Debug)]
struct AssetStorageMap {
    /// Map of `AssetType`s, keyed by asset data UUID.
    pub storages_by_data_uuid: HashMap<AssetTypeId, AssetType>,
    /// Map of `AssetType`s, keyed by asset type UUID.
    pub storages_by_asset_uuid: HashMap<AssetTypeId, AssetType>,
}

impl AssetStorageMap {
    /// Returns a new `AssetStorageMap`.
    pub fn new() -> AssetStorageMap {
        let mut storages_by_asset_uuid = HashMap::new();
        let mut storages_by_data_uuid = HashMap::new();
        for t in crate::inventory::iter::<AssetType> {
            storages_by_data_uuid.insert(t.data_uuid, t.clone());
            storages_by_asset_uuid.insert(t.asset_uuid, t.clone());
        }
        AssetStorageMap {
            storages_by_asset_uuid,
            storages_by_data_uuid,
        }
    }
}

impl Default for AssetStorageMap {
    fn default() -> Self {
        AssetStorageMap::new()
    }
}

/// Asset storage bridge between Amethyst and `atelier-assets`.
///
/// This type implements the `atelier_loader::AssetStorage` trait which `atelier_loader` uses as the
/// storage type for all assets of all asset types.
///
/// This contains immutable references to the `AssetStorageMap` and `World` resources.
struct WorldStorages<'a> {
    storage_map: &'a AssetStorageMap,
    res: &'a Resources,
}

impl<'a> WorldStorages<'a> {
    fn new(res: &'a Resources, storage_map: &'a AssetStorageMap) -> WorldStorages<'a> {
        WorldStorages { storage_map, res }
    }
}

impl<'a> atelier_loader::AssetStorage for WorldStorages<'a> {
    type HandleType = ();
    fn allocate(
        &self,
        asset_type: &AssetTypeId,
        _id: &AssetUuid,
        load_handle: &LoadHandle,
    ) -> Self::HandleType {
        let mut handle = None;
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &mut dyn AssetTypeStorage| {
            handle = Some(storage.allocate(load_handle));
        });
        handle.unwrap()
    }
    fn update_asset(
        &self,
        asset_type: &AssetTypeId,
        _handle: &Self::HandleType,
        data: &dyn AsRef<[u8]>,
        load_handle: &LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>> {
        use std::cell::RefCell; // can't move into closure, so we work around it with a RefCell + Option
        let moved_op = RefCell::new(Some(load_op));
        let mut result = None;
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &mut dyn AssetTypeStorage| {
            result = Some(storage.update_asset(
                load_handle,
                data,
                moved_op.replace(None).unwrap(),
                version,
            ));
        });
        result.unwrap()
    }
    fn commit_asset_version(
        &self,
        asset_type: &AssetTypeId,
        _handle: &Self::HandleType,
        load_handle: &LoadHandle,
        version: u32,
    ) {
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &mut dyn AssetTypeStorage| {
            storage.commit_asset_version(load_handle, version);
        });
    }
    fn free(
        &self,
        asset_type: &AssetTypeId,
        _storage_handle: Self::HandleType,
        load_handle: LoadHandle,
    ) {
        use std::cell::RefCell; // can't move into closure, so we work around it with a RefCell + Option
        let moved_handle = RefCell::new(Some(load_handle));
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.res, &mut |storage: &mut dyn AssetTypeStorage| {
            storage.free(moved_handle.replace(None).unwrap())
        });
    }
}

/// Registration information about an asset type.
#[derive(Clone)]
pub struct AssetType {
    /// UUID of the (de)serializable type representing the asset.
    pub data_uuid: AssetTypeId,
    /// UUID of the type representing the asset.
    pub asset_uuid: AssetTypeId,
    /// Function to create the `AssetTypeStorage`'s resources in the `World`.
    pub create_storage: fn(&mut Resources),
    /// Function that runs another function, passing in the `AssetTypeStorage`.
    pub with_storage: fn(&Resources, &mut dyn FnMut(&mut dyn AssetTypeStorage)),
}

impl std::fmt::Debug for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssetType {{ data_uuid: {:?}, asset_uuid: {:?} }}",
            self.data_uuid, self.asset_uuid
        )
    }
}

crate::inventory::collect!(AssetType);

/// Creates an `AssetType` to be stored in the `AssetType` `inventory`.
///
/// This function is not intended to be called be directly. Use the `register_asset_type!` macro
/// macro instead.
pub fn create_asset_type<Intermediate, Asset: 'static + TypeUuid + Send + Sync>() -> AssetType
where
    for<'a> Intermediate: 'static + Deserialize<'a> + TypeUuid + Send,
{
    AssetType {
        data_uuid: Intermediate::UUID,
        asset_uuid: Asset::UUID,
        create_storage: |res| {
            if res.try_fetch::<AssetStorage<Asset>>().is_none() {
                res.insert(AssetStorage::<Asset>::default())
            }
            if res.try_fetch::<ProcessingQueue<Intermediate>>().is_none() {
                res.insert(ProcessingQueue::<Intermediate>::default())
            }
        },
        with_storage: |res, func| {
            func(&mut (
                &*res.fetch::<ProcessingQueue<Intermediate>>(),
                &mut *res.fetch_mut::<AssetStorage<Asset>>(),
            ))
        },
    }
}

/// Registers an asset type which automatically prepares `AssetStorage` and `ProcessingQueue`.
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(Debug, TypeUuid)]
/// #[uuid = "28d51c52-be81-4d99-8cdc-20b26eb12448"]
/// pub struct MeshAsset {
///     buffer: (),
/// }
///
/// #[derive(Serialize, Deserialize, TypeUuid)]
/// #[uuid = "687b6d94-c653-4663-af73-e967c92ad140"]
/// pub struct VertexData {
///     positions: Vec<[f32; 3]>,
///     tex_coords: Vec<[f32; 2]>,
/// }
///
/// amethyst_assets::register_asset_type!(VertexData => MeshAsset);
/// ```
#[macro_export]
macro_rules! register_asset_type {
    ($intermediate:ty => $asset:ty) => {
        $crate::register_asset_type!(amethyst_assets; $intermediate => $asset);
    };
    ($krate:ident; $intermediate:ty => $asset:ty) => {
        $crate::inventory::submit!{
            #![crate = $krate]
            $crate::create_asset_type::<$intermediate, $asset>()
        }
    };
}
