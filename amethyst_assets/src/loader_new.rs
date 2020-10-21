use crate::{processor::ProcessingQueue, storage_new::AssetStorage};
use amethyst_core::ecs::{DispatcherBuilder, System, World};
// use atelier_assets::loader::{
//     handle::{self, AssetHandle, Handle, RefOp, WeakHandle},
//     storage::{DefaultIndirectionResolver, IndirectIdentifier, LoadStatus},
//     Loader, RpcIO,
// };
use atelier_loader::{
    // self,
    crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError},
    handle::{AssetHandle, GenericHandle, Handle, RefOp, SerdeContext, WeakHandle},
    storage::{AssetLoadOp, LoadInfo, LoaderInfoProvider},
    AssetTypeId,
    Loader as AtelierLoader,
};
use bincode;
use serde::de::Deserialize;
use std::{cell::RefCell, collections::HashMap, error::Error, sync::Arc};

pub(crate) use atelier_loader::LoadHandle;
pub use atelier_loader::{storage::LoadStatus, AssetUuid};
pub use type_uuid::TypeUuid;

/// Manages asset loading and storage for an application.
pub trait Loader: Send + Sync {
    /// Returns a generic asset handle and Loads the asset for the given UUID asynchronously.
    ///
    /// This is useful when loading an asset, but the asset's Rust type is unknown, such as for a
    /// loading screen that loads arbitrary assets.
    ///
    /// # Notes
    ///
    /// Be careful not to confuse `AssetUuid` with `AssetTypeId`:
    ///
    /// * `AssetUuid`: For an asset, such as "player_texture.png".
    /// * `AssetTypeId`: For an asset type, such as `Texture`.
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
    /// Be careful not to confuse `AssetUuid` with `AssetTypeId`:
    ///
    /// * `AssetUuid`: For an asset, such as "player_texture.png".
    /// * `AssetTypeId`: For an asset type, such as `Texture`.
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
    fn get_load_status_handle(&self, handle: LoadHandle) -> LoadStatus;

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

    /// Creates the `AssetTypeStorage`'s resources in the `World`.
    ///
    /// # Parameters
    ///
    /// * `world`: World in the application.
    fn init_world(&mut self, world: &mut World);

    /// Registers processing systems in the `Dispatcher`.
    ///
    /// # Parameters
    ///
    /// * `builder`: DispatcherBuilder in the application.
    fn init_dispatcher(&mut self, builder: &mut DispatcherBuilder<'static, 'static>);

    /// Updates asset loading state and removes assets that are no longer referenced.
    ///
    /// # Parameters
    ///
    /// * `world`: Specs world in the application.
    fn process(&mut self, world: &World) -> Result<(), Box<dyn Error>>;
}

/// Default loader is the Atelier Assets `RpcLoader`.
pub type DefaultLoader = LoaderWithStorage<atelier_loader::rpc_loader::RpcLoader>;

/// Asset loader and storage.
#[derive(Debug)]
pub struct LoaderWithStorage<T: AtelierLoader + Send + Sync> {
    loader: T,
    storage_map: AssetStorageMap,
    ref_sender: Arc<Sender<RefOp>>,
    ref_receiver: Receiver<RefOp>,
}

impl<T: AtelierLoader + Send + Sync + Default> Default for LoaderWithStorage<T> {
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
impl<T: AtelierLoader + Send + Sync> AtelierLoader for LoaderWithStorage<T> {
    fn add_ref(&self, id: AssetUuid) -> LoadHandle {
        self.loader.add_ref(id)
    }
    fn remove_ref(&self, load_handle: LoadHandle) {
        self.loader.remove_ref(load_handle)
    }
    fn get_asset(&self, load: LoadHandle) -> Option<(AssetTypeId, LoadHandle)> {
        self.loader.get_asset(load)
    }
    fn get_load(&self, id: AssetUuid) -> Option<LoadHandle> {
        self.loader.get_load(id)
    }
    fn get_load_info(&self, load: LoadHandle) -> Option<LoadInfo> {
        self.loader.get_load_info(load)
    }
    fn get_load_status(&self, load: LoadHandle) -> LoadStatus {
        self.loader.get_load_status(load)
    }
    fn process(
        &mut self,
        asset_storage: &dyn atelier_loader::storage::AssetStorage,
    ) -> Result<(), Box<dyn Error>> {
        self.loader.process(asset_storage)
    }
}

impl<T: AtelierLoader + Send + Sync> Loader for LoaderWithStorage<T> {
    fn load_asset_generic(&self, id: AssetUuid) -> GenericHandle {
        GenericHandle::new(self.ref_sender.clone(), self.loader.add_ref(id))
    }
    fn load_asset<A: TypeUuid>(&self, id: AssetUuid) -> Handle<A> {
        Handle::new(self.ref_sender.clone(), self.loader.add_ref(id))
    }
    fn get_load(&self, id: AssetUuid) -> Option<WeakHandle> {
        self.loader.get_load(id).map(|h| WeakHandle::new(h))
    }
    fn get_load_status_handle(&self, handle: LoadHandle) -> LoadStatus {
        self.loader.get_load_status(handle)
    }
    fn init_world(&mut self, world: &mut World) {
        for (_, storage) in self.storage_map.storages_by_asset_uuid.iter() {
            (storage.create_storage)(world);
        }
    }
    fn init_dispatcher(&mut self, builder: &mut DispatcherBuilder<'static, 'static>) {
        for (_, storage) in self.storage_map.storages_by_asset_uuid.iter() {
            (storage.register_system)(builder);
        }
    }

    fn process(&mut self, world: &World) -> Result<(), Box<dyn Error>> {
        loop {
            match self.ref_receiver.try_recv() {
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("RefOp receiver disconnected"),
                Ok(RefOp::Decrease(handle)) => self.loader.remove_ref(handle),
                Ok(RefOp::Increase(handle)) => {
                    self.loader
                        .get_load_info(handle)
                        .map(|info| self.loader.add_ref(info.asset_id));
                }
                Ok(RefOp::IncreaseUuid(uuid)) => {
                    self.loader.add_ref(uuid);
                }
            }
        }
        let storages = WorldStorages::new(world, &self.storage_map, &self.ref_sender);
        self.loader.process(&storages)
    }
}

/// Storage for a particular asset type.
///
/// This trait abtracts over the bridge between `atelier_loader` and Amethyst's asset storage. These
/// methods are called through dynamic dispatch by `atelier_loader` when an asset is loaded /
/// unloaded. All of these operations are performed on Amethyst's `AssetStorage`
pub trait AssetTypeStorage {
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
        handle: LoadHandle,
        data: &[u8],
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>>;

    /// Commits an asset.
    ///
    /// # Parameters
    ///
    /// * `handle`: Load handle of the asset.
    /// * `version`: Version of the asset -- this will be a new version for each hot reload.
    fn commit_asset_version(&mut self, handle: LoadHandle, version: u32);

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
    fn update_asset(
        &self,
        handle: LoadHandle,
        data: &[u8],
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>> {
        let asset = bincode::deserialize::<Intermediate>(data.as_ref())?;
        self.0.enqueue(handle, asset, load_op, version);
        Ok(())
    }
    fn commit_asset_version(&mut self, handle: LoadHandle, version: u32) {
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
    ref_sender: &'a Arc<Sender<RefOp>>,
    world: &'a World,
}

impl<'a> WorldStorages<'a> {
    fn new(
        world: &'a World,
        storage_map: &'a AssetStorageMap,
        ref_sender: &'a Arc<Sender<RefOp>>,
    ) -> WorldStorages<'a> {
        WorldStorages {
            storage_map,
            ref_sender,
            world,
        }
    }
}

impl<'a> atelier_loader::storage::AssetStorage for WorldStorages<'a> {
    fn update_asset(
        &self,
        loader_info: &dyn LoaderInfoProvider,
        asset_type: &AssetTypeId,
        data: &[u8],
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>> {
        // can't move into closure, so we work around it with a RefCell + Option
        let moved_op = RefCell::new(Some(load_op));
        let mut result = None;
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.world, &mut |storage: &mut dyn AssetTypeStorage| {
            SerdeContext::with(loader_info, self.ref_sender.clone(), || {
                result = Some(storage.update_asset(
                    load_handle,
                    data,
                    moved_op.replace(None).unwrap(),
                    version,
                ));
            });
        });
        result.unwrap()
    }
    fn commit_asset_version(
        &self,
        asset_type: &AssetTypeId,
        load_handle: LoadHandle,
        version: u32,
    ) {
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.world, &mut |storage: &mut dyn AssetTypeStorage| {
            storage.commit_asset_version(load_handle, version);
        });
    }
    fn free(&self, asset_type: &AssetTypeId, load_handle: LoadHandle) {
        // TODO: this RefCell dance is probably not needed
        // can't move into closure, so we work around it with a RefCell + Option
        let moved_handle = RefCell::new(Some(load_handle));
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.world, &mut |storage: &mut dyn AssetTypeStorage| {
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
    pub create_storage: fn(&mut World),
    pub register_system: fn(&mut DispatcherBuilder<'static, 'static>),
    /// Function that runs another function, passing in the `AssetTypeStorage`.
    pub with_storage: fn(&World, &mut dyn FnMut(&mut dyn AssetTypeStorage)),
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
pub fn create_asset_type<Intermediate, Asset, Processor>() -> AssetType
where
    Asset: 'static + TypeUuid + Send + Sync,
    for<'a> Processor: System<'a> + Send + Default + 'static,
    for<'a> Intermediate: 'static + Deserialize<'a> + TypeUuid + Send,
{
    AssetType {
        data_uuid: AssetTypeId(Intermediate::UUID),
        asset_uuid: AssetTypeId(Asset::UUID),
        create_storage: |res| {
            if res.try_fetch::<AssetStorage<Asset>>().is_none() {
                res.insert(AssetStorage::<Asset>::default())
            }
            if res.try_fetch::<ProcessingQueue<Intermediate>>().is_none() {
                res.insert(ProcessingQueue::<Intermediate>::default())
            }
        },
        register_system: |builder| builder.add(Processor::default(), "", &[]),
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
/// amethyst_assets::register_asset_type!(VertexData => MeshAsset; MeshProcessorSystem);
/// ```
#[macro_export]
macro_rules! register_asset_type {
    ($intermediate:ty => $asset:ty; $system:ty) => {
        $crate::register_asset_type!(amethyst_assets; $intermediate => $asset; $system);
    };
    ($krate:ident; $intermediate:ty => $asset:ty; $system:ty) => {
        $crate::inventory::submit!{
            #![crate = $krate]
            $crate::experimental::create_asset_type::<$intermediate, $asset, $system>()
        }
    };
}
