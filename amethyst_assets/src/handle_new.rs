use crate::{
    loader_new::{Loader, RefOp},
    storage_new::AssetStorage,
};
use amethyst_core::ecs::prelude::{Component, DenseVecStorage};
use atelier_loader::{self, LoaderInfoProvider};
use crossbeam_channel::{unbounded, Sender};
use derivative::Derivative;
use serde::{
    de::{self, Deserialize, Visitor},
    ser::{self, Serialize, Serializer},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
    sync::Arc,
};

pub(crate) use atelier_loader::LoadHandle;
pub use atelier_loader::{AssetRef, AssetUuid, LoadStatus, TypeUuid};

/// Keeps track of whether a handle ref is a strong, weak or "internal" ref
#[derive(Debug)]
enum HandleRefType {
    /// Strong references decrement the count on drop
    Strong(Arc<Sender<RefOp>>),
    /// Weak references do nothing on drop.
    Weak(Arc<Sender<RefOp>>),
    /// Internal references do nothing on drop, but turn into Strong references on clone.
    /// Should only be used for references stored in loaded assets to avoid self-referencing
    Internal(Arc<Sender<RefOp>>),
    /// Implementation detail, used when changing state in this enum
    None,
}

#[derive(Derivative)]
#[derivative(
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
struct HandleRef {
    id: LoadHandle,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    ref_type: HandleRefType,
}

impl Drop for HandleRef {
    fn drop(&mut self) {
        use HandleRefType::*;
        self.ref_type = match std::mem::replace(&mut self.ref_type, None) {
            Strong(sender) => {
                let _ = sender.send(RefOp::Decrease(self.id));
                Weak(sender)
            }
            r => r,
        };
    }
}

impl Clone for HandleRef {
    fn clone(&self) -> Self {
        use HandleRefType::*;
        Self {
            id: self.id,
            ref_type: match &self.ref_type {
                Internal(sender) | Strong(sender) => {
                    let _ = sender.send(RefOp::Increase(self.id));
                    Strong(sender.clone())
                }
                Weak(sender) => Weak(sender.clone()),
                None => panic!("unexpected ref type in clone()"),
            },
        }
    }
}

impl AssetHandle for HandleRef {
    fn load_handle(&self) -> LoadHandle {
        self.id
    }
}

/// Handle to an asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle<T: ?Sized> {
    handle_ref: HandleRef,
    marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// Creates a new handle with `HandleRefType::Strong`
    pub(crate) fn new(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self {
            handle_ref: HandleRef {
                id: handle,
                ref_type: HandleRefType::Strong(chan),
            },
            marker: PhantomData,
        }
    }

    /// Creates a new handle with `HandleRefType::Internal`
    pub(crate) fn new_internal(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self {
            handle_ref: HandleRef {
                id: handle,
                ref_type: HandleRefType::Internal(chan),
            },
            marker: PhantomData,
        }
    }
}

impl<T> AssetHandle for Handle<T> {
    fn load_handle(&self) -> LoadHandle {
        self.handle_ref.load_handle()
    }
}

/// Handle to an asset whose type is unknown during loading.
///
/// This is returned by `Loader::load_asset_generic` for assets loaded by UUID.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenericHandle {
    handle_ref: HandleRef,
}

impl GenericHandle {
    /// Creates a new handle with `HandleRefType::Strong`
    pub(crate) fn new(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self {
            handle_ref: HandleRef {
                id: handle,
                ref_type: HandleRefType::Strong(chan),
            },
        }
    }

    /// Creates a new handle with `HandleRefType::Internal`
    pub(crate) fn new_internal(chan: Arc<Sender<RefOp>>, handle: LoadHandle) -> Self {
        Self {
            handle_ref: HandleRef {
                id: handle,
                ref_type: HandleRefType::Internal(chan),
            },
        }
    }
}

impl AssetHandle for GenericHandle {
    fn load_handle(&self) -> LoadHandle {
        self.handle_ref.load_handle()
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

impl Component for GenericHandle {
    type Storage = DenseVecStorage<GenericHandle>;
}

impl Component for WeakHandle {
    type Storage = DenseVecStorage<WeakHandle>;
}

thread_local! {
    static LOADER: RefCell<Option<&'static dyn LoaderInfoProvider>> = RefCell::new(None);
    static REFOP_SENDER: RefCell<Option<Arc<Sender<RefOp>>>> = RefCell::new(None);
}

/// Used to make some limited Loader interactions available to `serde` Serialize/Deserialize
/// implementations by using thread-local storage. Required to support Serialize/Deserialize of Handle.
pub(crate) struct SerdeContext<'a> {
    loader: Option<&'static dyn LoaderInfoProvider>,
    sender: Option<Arc<Sender<RefOp>>>,
    _marker: PhantomData<&'a Self>,
}

impl<'a> SerdeContext<'a> {
    pub(crate) fn with_active<R>(
        f: impl FnOnce(Option<&&'static dyn LoaderInfoProvider>, Option<&Arc<Sender<RefOp>>>) -> R,
    ) -> R {
        LOADER.with(|l| REFOP_SENDER.with(|r| f(l.borrow().as_ref(), r.borrow().as_ref())))
    }
    pub(crate) fn with<F, R>(
        loader: &'a dyn LoaderInfoProvider,
        sender: Arc<Sender<RefOp>>,
        f: F,
    ) -> R
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

/// This context can be used to maintain AssetUuid references through a serialize/deserialize cycle
/// even if the LoadHandles produced are invalid. This is useful when a loader is not
/// present, such as when processing in the atelier-assets AssetDaemon.
struct DummySerdeContext {
    uuid_to_load: RefCell<HashMap<AssetRef, LoadHandle>>,
    load_to_uuid: RefCell<HashMap<LoadHandle, AssetRef>>,
    current_serde_dependencies: RefCell<HashSet<AssetRef>>,
    current_serde_asset: RefCell<Option<AssetUuid>>,
    ref_sender: Arc<Sender<RefOp>>,
    handle_gen: RefCell<u64>,
}

impl DummySerdeContext {
    pub fn new() -> Self {
        let (tx, _) = unbounded();
        Self {
            uuid_to_load: RefCell::new(HashMap::default()),
            load_to_uuid: RefCell::new(HashMap::default()),
            current_serde_dependencies: RefCell::new(HashSet::new()),
            current_serde_asset: RefCell::new(None),
            ref_sender: Arc::new(tx),
            handle_gen: RefCell::new(1),
        }
    }
}

impl LoaderInfoProvider for DummySerdeContext {
    fn get_load_handle(&self, asset_ref: &AssetRef) -> Option<LoadHandle> {
        let mut uuid_to_load = self.uuid_to_load.borrow_mut();
        let mut load_to_uuid = self.load_to_uuid.borrow_mut();
        Some(*uuid_to_load.entry(asset_ref.clone()).or_insert_with(|| {
            let mut handle_gen = self.handle_gen.borrow_mut();
            let new_id = *handle_gen + 1;
            *handle_gen += 1;
            let handle = LoadHandle(new_id);
            load_to_uuid.insert(handle, asset_ref.clone());
            handle
        }))
    }
    fn get_asset_id(&self, load: LoadHandle) -> Option<AssetUuid> {
        let maybe_asset = self.load_to_uuid.borrow().get(&load).map(|r| r.clone());
        if let Some(asset_ref) = maybe_asset.as_ref() {
            if let Some(current_serde_id) = &*self.current_serde_asset.borrow() {
                if AssetRef::Uuid(*current_serde_id) != *asset_ref
                    && *asset_ref != AssetRef::Uuid(AssetUuid::default())
                {
                    let mut dependencies = self.current_serde_dependencies.borrow_mut();
                    dependencies.insert(asset_ref.clone());
                }
            }
        }
        if let Some(AssetRef::Uuid(uuid)) = maybe_asset {
            Some(uuid)
        } else {
            None
        }
    }
}
struct DummySerdeContextHandle<'a> {
    dummy: Rc<DummySerdeContext>,
    ctx: SerdeContext<'a>,
}
impl<'a> atelier_importer::ImporterContextHandle for DummySerdeContextHandle<'a> {
    fn exit(&mut self) {
        SerdeContext::exit(&mut self.ctx)
    }
    fn enter(&mut self) {
        self.ctx = unsafe {
            let dummy_ref: &dyn LoaderInfoProvider = &*self.dummy;
            SerdeContext::enter(
                std::mem::transmute::<&dyn LoaderInfoProvider, &'static dyn LoaderInfoProvider>(
                    dummy_ref,
                ),
                self.dummy.ref_sender.clone(),
            )
        };
    }
    fn resolve_ref(&mut self, asset_ref: &AssetRef, asset: AssetUuid) {
        let new_ref = AssetRef::Uuid(asset);
        let mut uuid_to_load = self.dummy.uuid_to_load.borrow_mut();
        if let Some(handle) = uuid_to_load.get(asset_ref) {
            let handle = *handle;
            self.dummy
                .load_to_uuid
                .borrow_mut()
                .insert(handle, new_ref.clone());
            uuid_to_load.insert(new_ref, handle);
        }
    }
    /// Begin gathering dependencies for an asset
    fn begin_serialize_asset(&mut self, asset: AssetUuid) {
        let mut current = self.dummy.current_serde_asset.borrow_mut();
        if let Some(_) = &*current {
            panic!("begin_serialize_asset when current_serde_asset is already set");
        }
        *current = Some(asset);
    }
    /// Finish gathering dependencies for an asset
    fn end_serialize_asset(&mut self, _asset: AssetUuid) -> HashSet<AssetRef> {
        let mut current = self.dummy.current_serde_asset.borrow_mut();
        if let None = &*current {
            panic!("end_serialize_asset when current_serde_asset is not set");
        }
        *current = None;
        let mut deps = self.dummy.current_serde_dependencies.borrow_mut();
        std::mem::replace(&mut *deps, HashSet::new())
    }
}

struct DummySerdeContextProvider;
impl atelier_importer::ImporterContext for DummySerdeContextProvider {
    fn enter(&self) -> Box<dyn atelier_importer::ImporterContextHandle> {
        let dummy = Rc::new(DummySerdeContext::new());
        let dummy_ref: &dyn LoaderInfoProvider = &*dummy;
        // This should be an OwningRef
        let ctx = unsafe {
            SerdeContext::enter(std::mem::transmute(dummy_ref), dummy.ref_sender.clone())
        };
        Box::new(DummySerdeContextHandle { ctx, dummy })
    }
}
// Register the DummySerdeContextProvider as an ImporterContext to be used in atelier-assets.
inventory::submit!(atelier_importer::ImporterContextRegistration {
    instantiator: || Box::new(DummySerdeContextProvider),
});

fn serialize_handle<S>(load: LoadHandle, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    SerdeContext::with_active(|loader, _| {
        let loader = loader.expect("expected loader when serializing handle");
        use ser::SerializeSeq;
        let uuid: AssetUuid = loader.get_asset_id(load).unwrap_or(Default::default());
        let mut seq = serializer.serialize_seq(Some(uuid.0.len()))?;
        for element in &uuid.0 {
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
        serialize_handle(self.handle_ref.id, serializer)
    }
}
impl Serialize for GenericHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_handle(self.handle_ref.id, serializer)
    }
}

fn get_handle_ref(asset_ref: AssetRef) -> (LoadHandle, Arc<Sender<RefOp>>) {
    SerdeContext::with_active(|loader, sender| {
        let sender = sender
            .expect("no Sender<RefOp> set when deserializing handle")
            .clone();
        let handle = if asset_ref == AssetRef::Uuid(AssetUuid::default()) {
            LoadHandle(0)
        } else {
            loader
                .expect("no loader set in TLS when deserializing handle")
                .get_load_handle(&asset_ref)
                .unwrap_or_else(|| panic!("Handle for AssetUuid {:?} was not present when deserializing a Handle. This indicates missing dependency metadata, and can be caused by dependency cycles.", asset_ref))
        };
        let _ = sender.send(RefOp::Increase(handle));
        (handle, sender)
    })
}

impl<'de, T> Deserialize<'de> for Handle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Handle<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let asset_ref = if deserializer.is_human_readable() {
            deserializer.deserialize_any(AssetRefVisitor)?
        } else {
            deserializer.deserialize_seq(AssetRefVisitor)?
        };
        let (handle, sender) = get_handle_ref(asset_ref);
        Ok(Handle::new_internal(sender, handle))
    }
}

impl<'de> Deserialize<'de> for GenericHandle {
    fn deserialize<D>(deserializer: D) -> Result<GenericHandle, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let asset_ref = if deserializer.is_human_readable() {
            deserializer.deserialize_any(AssetRefVisitor)?
        } else {
            deserializer.deserialize_seq(AssetRefVisitor)?
        };
        let (handle, sender) = get_handle_ref(asset_ref);
        Ok(GenericHandle::new_internal(sender, handle))
    }
}

struct AssetRefVisitor;

impl<'de> Visitor<'de> for AssetRefVisitor {
    type Value = AssetRef;

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
        Ok(AssetRef::Uuid(AssetUuid(uuid)))
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use std::str::FromStr;
        match std::path::PathBuf::from_str(v) {
            Ok(path) => {
                if let Ok(uuid) = uuid::Uuid::parse_str(&path.to_string_lossy()) {
                    Ok(AssetRef::Uuid(AssetUuid(*uuid.as_bytes())))
                } else {
                    Ok(AssetRef::Path(path))
                }
            }
            Err(err) => Err(E::custom(format!(
                "failed to parse Handle string: {:?}",
                err
            ))),
        }
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
            Ok(AssetRef::Uuid(AssetUuid(a)))
        }
    }
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
