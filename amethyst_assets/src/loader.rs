use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use amethyst_core::{
    dispatcher::System,
    ecs::{DispatcherBuilder, Resources},
};
use amethyst_error::Error as AmethystError;
use atelier_assets::loader as atelier_loader;
pub(crate) use atelier_loader::LoadHandle;
use atelier_loader::{
    crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError},
    handle::{AssetHandle, GenericHandle, Handle, RefOp, SerdeContext, WeakHandle},
    storage::{
        AssetLoadOp, AtomicHandleAllocator, DefaultIndirectionResolver, HandleAllocator,
        IndirectIdentifier, IndirectionTable, LoaderInfoProvider,
    },
    AssetTypeId, Loader as AtelierLoader, RpcIO,
};
pub use atelier_loader::{storage::LoadStatus, AssetUuid};
use log::debug;
use serde::de::Deserialize;
pub use type_uuid::TypeUuid;

use crate::{processor::ProcessingQueue, progress::Progress, storage::AssetStorage, Asset};

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

    /// Returns an asset handle and Loads the asset for the given UUID asynchronously.
    ///
    /// This is useful when loading an asset whose Rust type is known.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Asset `TypeUuid`.
    fn load<T: TypeUuid>(&self, path: &str) -> Handle<T>;

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

    /// Load an asset from data and return a handle.
    fn load_from_data<A, P, D>(
        &self,
        data: D,
        progress: P,
        storage: &ProcessingQueue<D>,
    ) -> Handle<A>
    where
        A: Asset,
        P: Progress;

    // Creates the `AssetTypeStorage`'s resources in the `World`.
    ///
    /// # Parameters
    ///
    /// * `world`: World in the application.
    fn init_world(&mut self, resources: &mut Resources);

    /// Registers processing systems in the `Dispatcher`.
    ///
    /// # Parameters
    ///
    /// * `builder`: DispatcherBuilder in the application.
    fn init_dispatcher(&mut self, builder: &mut DispatcherBuilder);

    /// Updates asset loading state and removes assets that are no longer referenced.
    ///
    /// # Parameters
    ///
    /// * `world`: Specs world in the application.
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error + Send>>;
}

/// Default loader is the Atelier Assets `RpcLoader`.
pub type DefaultLoader = LoaderWithStorage;

/// Asset loader and storage.
pub struct LoaderWithStorage {
    loader: AtelierLoader,
    storage_map: AssetStorageMap,
    ref_sender: Sender<RefOp>,
    ref_receiver: Receiver<RefOp>,
    handle_allocator: Arc<AtomicHandleAllocator>,
    pub indirection_table: IndirectionTable,
}

impl Default for LoaderWithStorage {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let handle_allocator = Arc::new(AtomicHandleAllocator::default());
        let loader = AtelierLoader::new_with_handle_allocator(
            Box::new(RpcIO::default()),
            handle_allocator.clone(),
        );
        Self {
            indirection_table: loader.indirection_table(),
            loader,
            storage_map: Default::default(),
            ref_sender: tx,
            ref_receiver: rx,
            handle_allocator,
        }
    }
}

impl Loader for LoaderWithStorage {
    fn load_asset_generic(&self, id: AssetUuid) -> GenericHandle {
        GenericHandle::new(self.ref_sender.clone(), self.loader.add_ref(id))
    }
    fn load_asset<A: TypeUuid>(&self, id: AssetUuid) -> Handle<A> {
        Handle::new(self.ref_sender.clone(), self.loader.add_ref(id))
    }
    fn load<A: TypeUuid>(&self, path: &str) -> Handle<A> {
        Handle::new(
            self.ref_sender.clone(),
            self.loader
                .add_ref_indirect(IndirectIdentifier::Path(path.to_string())),
        )
    }
    fn get_load(&self, id: AssetUuid) -> Option<WeakHandle> {
        self.loader.get_load(id).map(WeakHandle::new)
    }
    fn get_load_status_handle(&self, handle: LoadHandle) -> LoadStatus {
        self.loader.get_load_status(handle)
    }

    /// Load an asset from data and return a handle.
    fn load_from_data<A, P, D>(
        &self,
        data: D,
        mut progress: P,
        processing_queue: &ProcessingQueue<D>,
    ) -> Handle<A>
    where
        A: Asset,
        P: Progress,
    {
        progress.add_assets(1);
        let tracker = progress.create_tracker();
        let tracker = Box::new(tracker);
        let handle = self.handle_allocator.alloc();
        let version = 0;
        processing_queue.enqueue_from_data(handle, data, tracker, version);
        Handle::<A>::new(self.ref_sender.clone(), handle)
    }

    fn init_world(&mut self, resources: &mut Resources) {
        for (_, storage) in self.storage_map.storages_by_asset_uuid.iter() {
            (storage.create_storage)(resources, &self.indirection_table);
        }
    }
    fn init_dispatcher(&mut self, builder: &mut DispatcherBuilder) {
        for (_, storage) in self.storage_map.storages_by_asset_uuid.iter() {
            (storage.register_system)(builder);
        }
    }

    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error + Send>> {
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
        let storages = WorldStorages::new(resources, &self.storage_map, &self.ref_sender);
        self.loader.process(&storages, &DefaultIndirectionResolver)
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
        data: std::vec::Vec<u8>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send>>;

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
    fn free(&mut self, handle: LoadHandle, version: u32);
}

impl<Intermediate, Asset: TypeUuid + Send + Sync> AssetTypeStorage
    for (&ProcessingQueue<Intermediate>, &mut AssetStorage<Asset>)
where
    for<'a> Intermediate: Deserialize<'a> + TypeUuid + Send,
{
    fn update_asset(
        &self,
        handle: LoadHandle,
        data: std::vec::Vec<u8>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send>> {
        debug!("AssetTypeStorage update_asset");
        match bincode::deserialize::<Intermediate>(data.as_ref()) {
            Err(err) => {
                debug!("Error in AssetTypeStorage deserialize");
                let e = AmethystError::from_string(format!("{}", err));
                load_op.error(err);
                Err(e.into_error())
            }
            Ok(asset) => {
                debug!("Ok in AssetTypeStorag deserialize");
                self.0.enqueue(handle, asset, load_op, version);
                Ok(())
            }
        }
    }
    fn commit_asset_version(&mut self, handle: LoadHandle, version: u32) {
        self.1.commit_asset(handle, version);
    }
    fn free(&mut self, handle: LoadHandle, version: u32) {
        self.1.remove_asset(handle, version);
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
    ref_sender: &'a Sender<RefOp>,
    resources: &'a Resources,
}

impl<'a> WorldStorages<'a> {
    fn new(
        resources: &'a Resources,
        storage_map: &'a AssetStorageMap,
        ref_sender: &'a Sender<RefOp>,
    ) -> WorldStorages<'a> {
        WorldStorages {
            storage_map,
            ref_sender,
            resources,
        }
    }
}

impl<'a> atelier_loader::storage::AssetStorage for WorldStorages<'a> {
    fn update_asset(
        &self,
        loader_info: &dyn LoaderInfoProvider,
        asset_type: &AssetTypeId,
        data: std::vec::Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send>> {
        debug!("update_asset");
        // FIXME
        // can't move into closure, so we work around it with a RefCell + Option
        let moved_op = RefCell::new(Some(load_op));
        let moved_data = RefCell::new(Some(data));
        let mut result = None;
        if let Some(asset_type) = self.storage_map.storages_by_data_uuid.get(asset_type) {
            (asset_type.with_storage)(self.resources, &mut |storage: &mut dyn AssetTypeStorage| {
                result = futures_executor::block_on(SerdeContext::with(
                    loader_info,
                    self.ref_sender.clone(),
                    async {
                        Some(storage.update_asset(
                            load_handle,
                            moved_data.replace(None).unwrap(),
                            moved_op.replace(None).unwrap(),
                            version,
                        ))
                    },
                ));
            });
        } else {
            log::warn!("Could not find AssetTypeID {}", asset_type);
            result = Some(Err(amethyst_error::Error::from_string(
                "Could not update asset.",
            )
            .into_error()))
        }
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
            .with_storage)(self.resources, &mut |storage: &mut dyn AssetTypeStorage| {
            storage.commit_asset_version(load_handle, version);
        });
    }
    fn free(&self, asset_type: &AssetTypeId, load_handle: LoadHandle, version: u32) {
        // TODO: this RefCell dance is probably not needed
        // can't move into closure, so we work around it with a RefCell + Option
        let moved_handle = RefCell::new(Some(load_handle));
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.resources, &mut |storage: &mut dyn AssetTypeStorage| {
            storage.free(moved_handle.replace(None).unwrap(), version)
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
    pub create_storage: fn(&mut Resources, &IndirectionTable),
    pub register_system: fn(&mut DispatcherBuilder),
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
pub fn create_asset_type<Intermediate, Asset, ProcessorSystem>() -> AssetType
where
    Asset: 'static + TypeUuid + Send + Sync,
    for<'a> Intermediate: 'static + Deserialize<'a> + TypeUuid + Send,
    ProcessorSystem: System<'static> + Default + 'static,
{
    log::debug!("Creating asset type: {:x?}", Asset::UUID);
    AssetType {
        data_uuid: AssetTypeId(Intermediate::UUID),
        asset_uuid: AssetTypeId(Asset::UUID),
        create_storage: |res, indirection_table| {
            debug!("Creating storage for {:x?}", Asset::UUID);
            res.get_or_insert_with(|| AssetStorage::<Asset>::new(indirection_table.clone()));
            debug!("Creating queue for intermediate {:x?}", Intermediate::UUID);
            res.get_or_insert_with(ProcessingQueue::<Intermediate>::default);
        },
        register_system: |builder| {
            builder.add_system(Box::new(ProcessorSystem::default()));
        },
        with_storage: |res, func| {
            func(&mut (
                res.get::<ProcessingQueue<Intermediate>>()
                    .expect("Could not get ProcessingQueue")
                    .deref(),
                res.get_mut::<AssetStorage<Asset>>()
                    .expect("Could not get_mut AssetStorage")
                    .deref_mut(),
            ))
        },
    }
}

/// Registers an asset type which automatically prepares `AssetStorage` and `ProcessingQueue`.
///
/// # Examples
///
/// ```rust,ignore
/// #[derive(TypeUuid)]
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
            $crate::create_asset_type::<$intermediate, $asset, $system>()
        }
    };
}
