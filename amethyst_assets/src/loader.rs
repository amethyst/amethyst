use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
};

use amethyst_core::{
    dispatcher::System,
    ecs::{DispatcherBuilder, Resources},
};
use amethyst_error::Error as AmethystError;
use distill::{
    importer::AssetMetadata, loader as distill_loader, loader::storage::IndirectionResolver,
};
pub(crate) use distill_loader::LoadHandle;
use distill_loader::{
    crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError},
    handle::{AssetHandle, GenericHandle, Handle, RefOp, SerdeContext, WeakHandle},
    storage::{
        AssetLoadOp, AtomicHandleAllocator, HandleAllocator, IndirectIdentifier, IndirectionTable,
        LoaderInfoProvider,
    },
    AssetTypeId, Loader as DistillLoader, RpcIO,
};
pub use distill_loader::{storage::LoadStatus, AssetUuid};
use log::debug;
use serde::de::Deserialize;

use crate::{
    processor::ProcessingQueue, progress::Progress, storage::AssetStorage, Asset, TypeUuid,
};

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
        self.get_load(id).map(|h| storage.get(&h)).flatten()
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

    /// Creates the `AssetTypeStorage`'s resources in the `World`.
    fn init_world(&mut self, resources: &mut Resources);

    /// Registers processing systems in the `Dispatcher`.
    fn init_dispatcher(&mut self, builder: &mut DispatcherBuilder);

    /// Updates asset loading state and removes assets that are no longer referenced.
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error + Send>>;
}

/// Asset loader and storage.
pub struct DefaultLoader {
    loader: DistillLoader,
    storage_map: AssetStorageMap,
    ref_sender: Sender<RefOp>,
    ref_receiver: Receiver<RefOp>,
    handle_allocator: Arc<AtomicHandleAllocator>,
    pub(crate) indirection_table: IndirectionTable,
}

impl Default for DefaultLoader {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let handle_allocator = Arc::new(AtomicHandleAllocator::default());
        let loader = DistillLoader::new_with_handle_allocator(
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

impl Loader for DefaultLoader {
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
                .add_ref_indirect(IndirectIdentifier::PathWithType(
                    path.to_string(),
                    AssetTypeId(A::UUID),
                )),
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
                Ok(RefOp::Decrease(handle)) => {
                    self.loader.remove_ref(handle);
                }
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
        self.loader.process(&storages, &AssetIndirectionResolver)
    }
}

pub struct AssetIndirectionResolver;
impl IndirectionResolver for AssetIndirectionResolver {
    fn resolve(
        &self,
        id: &IndirectIdentifier,
        candidates: Vec<(PathBuf, Vec<AssetMetadata>)>,
    ) -> Option<AssetUuid> {
        let id_type = id.type_id();
        for candidate in candidates {
            let candidate_assets_len = candidate.1.len();
            for asset in candidate.1 {
                if let Some(artifact) = asset.artifact {
                    if id_type.is_none()
                        || candidate_assets_len == 1
                        || *id_type.unwrap() == artifact.type_id
                    {
                        return Some(asset.id);
                    }
                }
            }
        }
        None
    }
}

/// Storage for a particular asset type.
///
/// This trait abtracts over the bridge between `distill_loader` and Amethyst's asset storage. These
/// methods are called through dynamic dispatch by `distill_loader` when an asset is loaded /
/// unloaded. All of these operations are performed on Amethyst's `AssetStorage`
pub trait AssetTypeStorage {
    /// Updates an asset.
    ///
    /// # Parameters
    ///
    /// * `handle`: Load handle of the asset.
    /// * `data`: Asset data bytes (uncompressed).
    /// * `load_op`: Load operation to notify `distill_loader` when the asset has been loaded.
    /// * `version`: Version of the asset -- this will be a new version for each hot reload.
    fn update_asset(
        &self,
        handle: LoadHandle,
        data: &[u8],
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

impl<Intermediate, Asset> AssetTypeStorage
    for (&ProcessingQueue<Intermediate>, &mut AssetStorage<Asset>)
where
    Asset: TypeUuid + Send + Sync,
    for<'a> Intermediate: Deserialize<'a> + TypeUuid + Send,
{
    fn update_asset(
        &self,
        handle: LoadHandle,
        data: &[u8],
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send>> {
        match bincode::deserialize::<Intermediate>(data) {
            Err(err) => {
                let e = AmethystError::from_string(format!("{}", err));
                load_op.error(err);
                Err(e.into_error())
            }
            Ok(asset) => {
                self.0.enqueue(handle, asset, load_op, version);
                Ok(())
            }
        }
    }

    fn commit_asset_version(&mut self, handle: LoadHandle, version: u32) {
        self.1.commit_asset(handle, version);
        self.0.enqueue_changed(handle);
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

/// Asset storage bridge between Amethyst and `distill`.
///
/// This type implements the `distill_loader::AssetStorage` trait which `distill_loader` uses as the
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

impl<'a> distill_loader::storage::AssetStorage for WorldStorages<'a> {
    fn update_asset(
        &self,
        loader_info: &dyn LoaderInfoProvider,
        asset_type: &AssetTypeId,
        data: Vec<u8>,
        load_handle: LoadHandle,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error + Send>> {
        let moved_op = RefCell::new(Some(load_op));
        let mut result = None;
        if let Some(asset_type) = self.storage_map.storages_by_data_uuid.get(asset_type) {
            (asset_type.with_storage)(self.resources, &mut |storage: &mut dyn AssetTypeStorage| {
                futures_executor::block_on(SerdeContext::with(
                    loader_info,
                    self.ref_sender.clone(),
                    async {
                        result = Some(storage.update_asset(
                            load_handle,
                            &data,
                            moved_op.replace(None).unwrap(),
                            version,
                        ))
                    },
                ))
            });
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
        (self
            .storage_map
            .storages_by_data_uuid
            .get(asset_type)
            .expect("could not find asset type")
            .with_storage)(self.resources, &mut |storage: &mut dyn AssetTypeStorage| {
            storage.free(load_handle, version)
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
    ProcessorSystem: System + Default + 'static,
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
            builder.add_system(ProcessorSystem::default());
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
/// ```
/// use serde::{Deserialize, Serialize};
/// use amethyst::assets::{register_asset_type, Asset, AssetProcessorSystem, AssetStorage, LoadHandle, ProcessableAsset, ProcessingState, TypeUuid};
/// use amethyst::error::Error;
///
/// #[derive(Serialize, Deserialize, TypeUuid)]
/// #[uuid = "00000000-0000-0000-0000-000000000000"]
/// pub struct VertexData {
///     buffer: Vec<u8>,
/// }
///
/// impl Asset for Vertex {
///     fn name() -> &'static str {
///         "Vertex"
///     }
///     type Data = VertexData;
/// }
///
/// #[derive(Default, TypeUuid)]
/// #[uuid = "00000000-0000-0000-0000-000000000001"]
/// pub struct Vertex {
///     positions: Vec<[f32; 3]>,
///     tex_coords: Vec<[f32; 2]>,
/// }
///
/// register_asset_type!(VertexData => Vertex; AssetProcessorSystem<Vertex>);
///
/// impl ProcessableAsset for Vertex {
///    fn process(
///        data: VertexData,
///        _storage: &mut AssetStorage<Vertex>,
///        _handle: &LoadHandle,
///    ) -> Result<ProcessingState<VertexData, Vertex>, Error> {
///        log::debug!("Loading Vertex");
///        Ok(ProcessingState::Loaded(Vertex::default()))
///    }
/// }
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
