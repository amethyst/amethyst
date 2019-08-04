use crate::{
    loader_new::{Loader, RefOp},
    storage_new::AssetStorage,
};
use amethyst_core::ecs::{
    prelude::{Component, DenseVecStorage},
};
use atelier_loader::{self, LoaderInfoProvider};
use ccl::dhashmap::DHashMap;
use crossbeam::channel::{unbounded, Sender};
use derivative::Derivative;
use serde::{
    de::{self, Deserialize, Visitor},
    ser::{self, Serialize, Serializer},
};
use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

pub(crate) use atelier_loader::LoadHandle;
pub use atelier_loader::{AssetUuid, LoadStatus, TypeUuid};

thread_local! {
    static LOADER: RefCell<Option<&'static dyn LoaderInfoProvider>> = RefCell::new(None);
    static REFOP_SENDER: RefCell<Option<Arc<Sender<RefOp>>>> = RefCell::new(None);
}

pub(crate) struct SerdeContext<'a> {
    loader: Option<&'static dyn LoaderInfoProvider>,
    sender: Option<Arc<Sender<RefOp>>>,
    _marker: PhantomData<&'a Self>,
}

impl<'a> SerdeContext<'a> {
    pub(crate) fn with_active<R>(f: impl FnOnce(Option<&&'static dyn LoaderInfoProvider>, Option<&Arc<Sender<RefOp>>>) -> R) -> R {
        LOADER.with(|l| {
            REFOP_SENDER.with(|r| {
                f(l.borrow().as_ref(), r.borrow().as_ref())
            })
        })
    }
    pub(crate) fn with<F, R>(loader: &'a dyn LoaderInfoProvider, sender: Arc<Sender<RefOp>>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let mut ctx = Self::enter(loader, sender);
        let result = f();
        ctx.exit();
        result
    }
    fn enter(loader: &'a dyn LoaderInfoProvider, sender: Arc<Sender<RefOp>>) -> Self {
        // The loader lifetime needs to be transmuted to 'static to be able to be stored in the static.
        // This is safe since SerdeContext's lifetime cannot be longer than 'a due to its marker,
        // and the reference in the static is removed in `Drop`.
        let loader = unsafe {
            LOADER.with(|l| {
                l.replace(Some(std::mem::transmute::<
                    &dyn LoaderInfoProvider,
                    &'static dyn LoaderInfoProvider,
                >(loader)))
            })
        };
        let sender = REFOP_SENDER.with(|r| r.replace(Some(sender)));
        Self {
            loader,
            sender,
            _marker: PhantomData,
        }
    }
    fn exit(&mut self) {
        // restore the previous loader & sender
        LOADER.with(|l| l.replace(self.loader.take()));
        REFOP_SENDER.with(|r| r.replace(self.sender.take()));
    }
}

impl<'a> Drop for SerdeContext<'a> {
    fn drop(&mut self) {
        if self.loader.is_some() {
            LOADER.with(|l| l.replace(self.loader.take()));
        }
        if self.sender.is_some() {
            REFOP_SENDER.with(|r| r.replace(self.sender.take()));
        }
    }
}

/// This context can be used to maintain values through a serialize/deserialize cycle
/// even if the LoadHandles produced are invalid. This is useful when a loader is not
/// present, such as when processing in the atelier-assets AssetDaemon.
struct DummySerdeContext {
    uuid_to_load: DHashMap<AssetUuid, LoadHandle>,
    load_to_uuid: DHashMap<LoadHandle, AssetUuid>,
    ref_sender: Arc<Sender<RefOp>>,
    handle_gen: AtomicU64,
}

impl DummySerdeContext {
    pub fn new() -> Self {
        let (tx, _) = unbounded();
        Self {
            uuid_to_load: DHashMap::default(),
            load_to_uuid: DHashMap::default(),
            ref_sender: Arc::new(tx),
            handle_gen: AtomicU64::new(1),
        }
    }
}

impl LoaderInfoProvider for DummySerdeContext {
    fn get_load_handle(&self, id: AssetUuid) -> Option<LoadHandle> {
        self.uuid_to_load.get(&id).map(|l| *l)
    }
    fn get_asset_id(&self, load: LoadHandle) -> Option<AssetUuid> {
        self.load_to_uuid.get(&load).map(|l| *l)
    }
    fn add_ref(&self, id: AssetUuid) -> LoadHandle {
        *self.uuid_to_load.get_or_insert_with(&id, || {
            let handle = LoadHandle(self.handle_gen.fetch_add(1, Ordering::Relaxed));
            self.load_to_uuid.insert(handle, id);
            handle
        })
    }
}
struct DummySerdeContextHandle<'a> {
    _dummy: Arc<DummySerdeContext>,
    ctx: SerdeContext<'a>,
}
impl<'a> atelier_importer::ImporterContextHandle for DummySerdeContextHandle<'a> {
    fn exit(&mut self) {
        SerdeContext::exit(&mut self.ctx)
    }
}

struct DummySerdeContextProvider(Arc<DummySerdeContext>);
impl atelier_importer::ImporterContext for DummySerdeContextProvider {
    fn enter(&self) -> Box<dyn atelier_importer::ImporterContextHandle> {
        let dummy = self.0.clone();
        let dummy_ref: &dyn LoaderInfoProvider = &*dummy;
        // This should be an OwningRef
        let ctx = unsafe {
            SerdeContext::enter(std::mem::transmute(dummy_ref), self.0.ref_sender.clone())
        };
        Box::new(DummySerdeContextHandle { ctx, _dummy: dummy })
    }
}

inventory::submit!(atelier_importer::ImporterContextRegistration {
    instantiator: || Box::new(DummySerdeContextProvider(
        Arc::new(DummySerdeContext::new())
    )),
});

/// Handle to an asset.
#[derive(Derivative)]
#[derivative(
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
    pub(crate) fn new(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
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
    fn load_handle(&self) -> LoadHandle {
        self.id
    }
}

fn serialize_handle<S>(load: LoadHandle, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    SerdeContext::with_active(|loader, _| {
        let loader = loader.expect("expected loader when serializing handle");
        use ser::SerializeSeq;
        let uuid: AssetUuid = loader.get_asset_id(load)
            .unwrap_or(Default::default());
        let mut seq = serializer.serialize_seq(Some(uuid.len()))?;
        for element in &uuid {
            seq.serialize_element(element)?;
        }
        seq.end()
    })
}
impl<T> Serialize for Handle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_handle(self.id, serializer)
    }
}
impl Serialize for GenericHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_handle(self.id, serializer)
    }
}

fn add_uuid_handle_ref(uuid: AssetUuid) -> (LoadHandle, Arc<Sender<RefOp>>) {
    SerdeContext::with_active(|loader, sender| {
        let sender = sender
            .expect("no Sender<RefOp> set when deserializing handle").clone();
        sender.send(RefOp::Increase(uuid));
        let handle = loader
            .expect("no loader set in TLS when deserializing handle")
            .add_ref(uuid);
        (handle, sender)
    })
}

impl<'de, T> Deserialize<'de> for Handle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Handle<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let uuid = deserializer.deserialize_seq(AssetUuidVisitor)?;
        let (handle, sender) = add_uuid_handle_ref(uuid);
        Ok(Handle::new(sender, handle))
    }
}

impl<'de> Deserialize<'de> for GenericHandle {
    fn deserialize<D>(deserializer: D) -> Result<GenericHandle, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let uuid = deserializer.deserialize_seq(AssetUuidVisitor)?;
        let (handle, sender) = add_uuid_handle_ref(uuid);
        Ok(GenericHandle::new(sender, handle))
    }
}

struct AssetUuidVisitor;

impl<'de> Visitor<'de> for AssetUuidVisitor {
    type Value = AssetUuid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an array of 16 u8")
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        use de::Error;
        let mut uuid: [u8; 16] = Default::default();
        for i in 0..16 {
            if let Some(byte) = seq.next_element::<u8>()? {
                uuid[i] = byte;
            } else {
                return Err(A::Error::custom(format!(
                    "expected byte at element {} when deserializing handle",
                    i
                )));
            }
        }
        if let Some(_) = seq.next_element::<u8>()? {
            return Err(A::Error::custom(
                "too many elements when deserializing handle",
            ));
        }
        Ok(uuid)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() != 16 {
            Err(E::custom(format!(
                "byte array len == {}, expected {}",
                v.len(),
                16
            )))
        } else {
            let mut a = <[u8; 16]>::default();
            a.copy_from_slice(v);
            Ok(a)
        }
    }
}

/// Handle to an asset whose type is unknown during loading.
///
/// This is returned by `Loader::load_asset_generic` for assets loaded by UUID.
#[derive(Derivative)]
#[derivative(
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
    pub(crate) fn new(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self { chan, id: handle }
    }
}

impl Drop for GenericHandle {
    fn drop(&mut self) {
        self.chan.send(RefOp::Decrease(self.id))
    }
}

impl AssetHandle for GenericHandle {
    fn load_handle(&self) -> LoadHandle {
        self.id
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
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
pub struct WeakHandle {
    id: LoadHandle,
}

impl WeakHandle {
    pub(crate) fn new(handle: LoadHandle) -> Self {
        WeakHandle { id: handle }
    }
}

impl AssetHandle for WeakHandle {
    fn load_handle(&self) -> LoadHandle {
        self.id
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
        WeakHandle::new(self.load_handle())
    }

    /// Returns the `LoadHandle` of this asset handle.
    fn load_handle(&self) -> LoadHandle;
}
