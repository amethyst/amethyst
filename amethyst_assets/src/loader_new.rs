use crate::{
    storage_new::AssetStorage,
    processor::{ProcessingQueue},
    Asset,
};
use amethyst_core::ecs::{
    prelude::{Component, DenseVecStorage},
    storage::UnprotectedStorage,
    Resources,
};
use atelier_loader::{self, AssetLoadOp, AssetTypeId, Loader as AtelierLoader};
use bincode;
use crossbeam::channel::{unbounded, Receiver, Sender};
use derivative::Derivative;
use serde::de::Deserialize;
use std::{collections::HashMap, error::Error, marker::PhantomData, sync::Arc};

pub(crate) use atelier_loader::LoadHandle;
pub use atelier_loader::{TypeUuid, LoadStatus, AssetUuid};

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
    fn get_load_handle(&self) -> &LoadHandle {
        &self.id
    }
}
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
    fn get_load_handle(&self) -> &LoadHandle {
        &self.id
    }
}

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
    fn get_load_handle(&self) -> &LoadHandle {
        &self.id
    }
}

impl<T: TypeUuid + Send + Sync + 'static> Component for Handle<T> {
    type Storage = DenseVecStorage<Handle<T>>;
}

pub trait AssetHandle {
    fn get_load_status<T: Loader>(&self, loader: &T) -> LoadStatus {
        loader.get_load_status_handle(self.get_load_handle())
    }
    fn get_asset<'a, T: Asset + TypeUuid>(&self, storage: &'a AssetStorage<T>) -> Option<&'a T>
    where
        Self: Sized,
    {
        storage.get(self)
    }
    fn get_asset_mut<'a, T: Asset + TypeUuid>(
        &self,
        storage: &'a mut AssetStorage<T>,
    ) -> Option<&'a mut T>
    where
        Self: Sized,
    {
        storage.get_mut(self)
    }
    fn get_version<'a, T: Asset + TypeUuid>(&self, storage: &'a AssetStorage<T>) -> Option<u32>
    where
        Self: Sized,
    {
        storage.get_version(self)
    }
    fn get_asset_with_version<'a, T: Asset + TypeUuid>(
        &self,
        storage: &'a AssetStorage<T>,
    ) -> Option<(&'a T, u32)>
    where
        Self: Sized,
    {
        storage.get_asset_with_version(self)
    }
    /// Downgrades the handle and creates a `WeakHandle`.
    fn downgrade(&self) -> WeakHandle {
        WeakHandle::new(*self.get_load_handle())
    }
    fn get_load_handle(&self) -> &LoadHandle;
}

pub trait Loader: Send + Sync {
    fn load_asset_generic(&self, id: AssetUuid) -> GenericHandle;
    fn load_asset<T: TypeUuid>(&self, id: AssetUuid) -> Handle<T>;
    fn get_load(&self, id: AssetUuid) -> Option<WeakHandle>;
    fn get_load_status(&self, id: AssetUuid) -> LoadStatus {
        self.get_load(id)
            .map(|h| self.get_load_status_handle(h.get_load_handle()))
            .unwrap_or(LoadStatus::NotRequested)
    }
    fn get_load_status_handle(&self, handle: &LoadHandle) -> LoadStatus;
    fn get_asset<'a, T: Asset + TypeUuid>(
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
    fn get_asset_mut<'a, T: Asset + TypeUuid>(
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
    fn init_world(&mut self, resources: &mut Resources);
    fn process(&mut self, resources: &Resources) -> Result<(), Box<dyn Error>>;
}

pub type DefaultLoader = LoaderWithStorage<atelier_loader::rpc_loader::RpcLoader<()>>;
enum RefOp {
    Decrease(LoadHandle),
}
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

pub trait AssetTypeStorage {
    fn allocate(&self, handle: &LoadHandle) -> ();
    fn update_asset(
        &self,
        handle: &LoadHandle,
        data: &dyn AsRef<[u8]>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>>;
    fn commit_asset_version(&mut self, handle: &LoadHandle, version: u32);
    fn free(&mut self, handle: LoadHandle);
}
impl<A: Asset> AssetTypeStorage for (&ProcessingQueue<A::Data>, &mut AssetStorage<A>)
where
    for<'a> A::Data: Deserialize<'a> + TypeUuid,
{
    fn allocate(&self, load_handle: &LoadHandle) -> () {}
    fn update_asset(
        &self,
        handle: &LoadHandle,
        data: &dyn AsRef<[u8]>,
        load_op: AssetLoadOp,
        version: u32,
    ) -> Result<(), Box<dyn Error>> {
        let asset = bincode::deserialize::<A::Data>(data.as_ref())?;
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

#[derive(Debug)]
struct AssetStorageMap {
    pub storages_by_data_uuid: HashMap<AssetTypeId, AssetType>,
    pub storages_by_asset_uuid: HashMap<AssetTypeId, AssetType>,
}

impl AssetStorageMap {
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
        id: &AssetUuid,
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
        handle: &Self::HandleType,
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
        handle: &Self::HandleType,
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
        storage_handle: Self::HandleType,
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
#[derive(Clone)]
pub struct AssetType {
    pub data_uuid: AssetTypeId,
    pub asset_uuid: AssetTypeId,
    pub create_storage: fn(&mut Resources),
    pub with_storage: fn(&Resources, &mut dyn FnMut(&mut dyn AssetTypeStorage)),
}
impl std::fmt::Debug for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssetType {{ data_uuid: {:?}, asset_uud: {:?} }}",
            self.data_uuid, self.asset_uuid
        )
    }
}
crate::inventory::collect!(AssetType);

pub fn create_asset_type<A: Asset + TypeUuid>() -> AssetType
where
    for<'a> A::Data: Deserialize<'a> + TypeUuid,
{
    AssetType {
        data_uuid: A::Data::UUID,
        asset_uuid: A::UUID,
        create_storage: |res| {
            if res.try_fetch::<AssetStorage<A>>().is_none() {
                res.insert(AssetStorage::<A>::default())
            }
            if res.try_fetch::<ProcessingQueue<A::Data>>().is_none() {
                res.insert(ProcessingQueue::<A::Data>::default())
            }
        },
        with_storage: |res, func| {
            func(&mut (
                &*res.fetch::<ProcessingQueue<A::Data>>(),
                &mut *res.fetch_mut::<AssetStorage<A>>(),
            ))
        },
    }
}

#[macro_export]
macro_rules! asset_type {
    ($($intermediate:ty => $asset:ty),*,) => {
        $crate::asset_type! {
            $(
                $intermediate => $asset
            ),*
        }
    };
    ($($intermediate:ty => $asset:ty),*) => {
            use $crate::inventory;
            use $crate::create_asset_type;
            use super::*;
            $(
                inventory::submit! {
                    create_asset_type::<$asset>()
                }
            )*
    }
}
