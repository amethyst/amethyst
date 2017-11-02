use std::hash;
use std::marker::PhantomData;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};

use amethyst_core::Time;
use crossbeam::sync::MsQueue;
use hibitset::BitSet;
use fnv::FnvHashMap;
use rayon::ThreadPool;
use specs::{Component, Fetch, FetchMut, System, UnprotectedStorage, VecStorage};

use asset::{Asset, FormatValue};
use error::{ErrorKind, Result, ResultExt};
use progress::Tracker;
use reload::{HotReloadStrategy, Reload};

/// An `Allocator`, holding a counter for producing unique IDs.
#[derive(Debug, Default)]
pub struct Allocator {
    count: AtomicUsize,
}

impl Allocator {
    /// Produces a new id.
    pub fn next_id(&self) -> usize {
        self.count.fetch_add(1, Ordering::Relaxed)
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
struct AllocStorage<A> {
    assets: VecStorage<A>,
    bitset: BitSet,
}

impl<A> AllocStorage<A> {
    fn get(&self, h: &HandleAlloc<A>) -> Option<&A> {
        if self.bitset.contains(h.id()) {
            Some(unsafe { self.assets.get(h.id()) })
        } else {
            None
        }
    }

    fn get_mut(&mut self, h: &HandleAlloc<A>) -> Option<&mut A> {
        if self.bitset.contains(h.id()) {
            Some(unsafe { self.assets.get_mut(h.id()) })
        } else {
            None
        }
    }

    fn insert(&mut self, h: &HandleAlloc<A>, val: A) {
        let id = h.id();
        // NOTE: the loader has to ensure that a handle will be used
        // together with a `Data` only once.
        debug_assert!(
            !self.bitset.contains(id),
            "Insertion has already been made!"
        );
        unsafe {
            self.assets.insert(id, val);
        }
        self.bitset.add(id);
    }

    fn remove(&mut self, h: &HandleAlloc<A>) -> A {
        let id = h.id();
        debug_assert!(
            self.bitset.contains(id),
            "Can't remove asset which wasn't inserted"
        );
        self.bitset.remove(id);

        unsafe { self.assets.remove(id) }
    }
}

impl<A> Drop for AllocStorage<A> {
    fn drop(&mut self) {
        let bitset = &self.bitset;
        unsafe { self.assets.clean(|id| bitset.contains(id)) }
    }
}

/// An asset storage, storing the actual assets and allocating
/// handles to them.
pub struct AssetStorage<A: Asset> {
    alloc_storage: AllocStorage<A>,
    handles: Vec<HandleAlloc<A>>,
    handle_alloc: Allocator,
    library_storage: LibraryStorage<A>,
    pub(crate) processed: Arc<MsQueue<Processed<A>>>,
    reloads: Vec<(WeakHandle<A>, Box<Reload<A>>)>,
    unused_handles: MsQueue<HandleAlloc<A>>,
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }

    /// Allocate a new handle.
    pub(crate) fn allocate(&self) -> HandleAlloc<A> {
        self.unused_handles
            .try_pop()
            .unwrap_or_else(|| self.allocate_new())
    }

    fn allocate_new(&self) -> HandleAlloc<A> {
        let id = self.handle_alloc.next_id() as u32;
        let handle = HandleAlloc {
            id: Arc::new(id),
            marker: PhantomData,
        };

        handle
    }

    /// When cloning an asset handle, you'll get another handle,
    /// but pointing to the same asset. If you instead want to
    /// indeed create a new asset, you can use this method.
    /// Note however, that it needs a mutable borrow of `self`,
    /// so it can't be used in parallel.
    pub fn clone_asset(&mut self, handle: &Handle<A>) -> Option<Handle<A>>
    where
        A: Clone,
    {
        //        if let Some(asset) = self.get(handle).map(A::clone) {
        //            let h = self.allocate();
        //
        //            let id = h.id();
        //            self.bitset.add(id);
        //            self.handles.push(h.clone());
        //
        //            unsafe {
        //                self.assets.insert(id, asset);
        //            }
        //
        //            Some(h)
        //        } else {
        //            None
        //        }
        unimplemented!()
    }

    pub fn desc_storage(&self) -> DescStorage<A> {
        unimplemented!()
    }

    /// Get an asset from a given asset handle.
    pub fn get(&self, handle: &Handle<A>) -> Option<&A> {
        match handle.inner {
            HandleInner::Alloc(ref h) => self.alloc_storage.get(h),
            HandleInner::Library(ref h) => self.library_storage
                .get(h)
                .and_then(move |h| self.alloc_storage.get(h)),
        }
    }

    /// Get an asset mutably from a given asset handle.
    pub fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A> {
        let alloc = &mut self.alloc_storage;

        match handle.inner {
            HandleInner::Alloc(ref h) => alloc.get_mut(h),
            HandleInner::Library(ref h) => self.library_storage
                .get(h)
                .and_then(move |h| alloc.get_mut(h)),
        }
    }

    /// Process finished asset data and maintain the storage.
    pub fn process<F>(
        &mut self,
        f: F,
        frame_number: u64,
        pool: &ThreadPool,
        strategy: Option<&HotReloadStrategy>,
    ) where
        F: FnMut(A::Data) -> Result<A>,
    {
        self.process_custom_drop(f, |_| {}, frame_number, pool, strategy);
    }

    /// Process finished asset data and maintain the storage.
    /// This calls the `drop_fn` closure for assets that were removed from the storage.
    pub fn process_custom_drop<F, D>(
        &mut self,
        mut f: F,
        mut drop_fn: D,
        frame_number: u64,
        pool: &ThreadPool,
        strategy: Option<&HotReloadStrategy>,
    ) where
        D: FnMut(A),
        F: FnMut(A::Data) -> Result<A>,
    {
        while let Some(processed) = self.processed.try_pop() {
            let alloc = &mut self.alloc_storage;
            let handles = &mut self.handles;
            let reloads = &mut self.reloads;

            let f = &mut f;
            let (reload_obj, handle) = match processed {
                Processed::NewAsset {
                    data,
                    handle,
                    name,
                    tracker,
                } => {
                    let (asset, reload_obj) = match data.map(
                        |FormatValue { data, reload }| (data, reload),
                    ).and_then(|(d, rel)| f(d).map(|a| (a, rel)))
                        .chain_err(|| ErrorKind::Asset(name))
                    {
                        Ok(x) => {
                            tracker.success();

                            x
                        }
                        Err(e) => {
                            tracker.fail(e);

                            continue;
                        }
                    };

                    alloc.insert(&handle, asset);
                    handles.push(handle.clone());

                    (reload_obj, handle.into())
                }
                Processed::HotReload {
                    data,
                    handle,
                    name,
                    old_reload,
                } => {
                    let (asset, reload_obj) = match data.map(
                        |FormatValue { data, reload }| (data, reload),
                    ).and_then(|(d, rel)| f(d).map(|a| (a, rel)))
                        .chain_err(|| ErrorKind::Asset(name))
                    {
                        Ok(x) => x,
                        Err(e) => {
                            eprintln!("Failed to hot-reload: {}", e);

                            reloads.push((handle.downgrade(), old_reload));

                            continue;
                        }
                    };

                    match handle.inner {
                        HandleInner::Alloc(ref h) => {
                            *alloc.get_mut(h).unwrap() = asset;
                        }
                        _ => unimplemented!(),
                    }

                    (reload_obj, handle)
                }
            };

            // Add the reload obj if it is `Some`.
            if let Some(reload_obj) = reload_obj {
                reloads.push((handle.downgrade(), reload_obj));
            }
        }

        let mut skip = 0;
        while let Some(i) = self.handles
            .iter()
            .skip(skip)
            .position(HandleAlloc::is_unique)
        {
            skip = i;
            let handle = self.handles.swap_remove(i);
            drop_fn(self.alloc_storage.remove(&handle));

            // Can't reuse old handle here, because otherwise weak handles would still be valid.
            // TODO: maybe just store u32?
            self.unused_handles.push(HandleAlloc {
                id: Arc::new(handle.id()),
                marker: PhantomData,
            });
        }

        if strategy
            .map(|s| s.needs_reload(frame_number))
            .unwrap_or(false)
        {
            self.hot_reload(pool);
        }
    }

    fn hot_reload(&mut self, pool: &ThreadPool) {
        self.reloads.retain(|&(ref handle, _)| !handle.is_dead());
        while let Some(p) = self.reloads
            .iter()
            .position(|&(_, ref rel)| rel.needs_reload())
        {
            let (handle, rel) = self.reloads.swap_remove(p);

            if let Some(handle) = handle.upgrade() {
                let processed = self.processed.clone();
                pool.spawn(move || {
                    let old_reload = rel.clone();
                    let name = rel.name();
                    let format = rel.format();
                    let data = rel.reload().chain_err(|| ErrorKind::Format(format));

                    let p = Processed::HotReload {
                        data,
                        name,
                        handle,
                        old_reload,
                    };
                    processed.push(p);
                });
            }
        }
    }
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            alloc_storage: Default::default(),
            handles: Default::default(),
            handle_alloc: Default::default(),
            library_storage: Default::default(),
            processed: Arc::new(MsQueue::new()),
            reloads: Default::default(),
            unused_handles: MsQueue::new(),
        }
    }
}

pub struct DescStorage<A> {
    marker: PhantomData<A>,
}

/// A default implementation for an asset processing system
/// which converts data to assets and maintains the asset storage
/// for `A`.
///
/// This system can only be used if the asset data implements
/// `Into<Result<A, BoxedErr>>`.
pub struct Processor<A> {
    marker: PhantomData<A>,
}

impl<A> Processor<A> {
    /// Creates a new asset processor for
    /// assets of type `A`.
    pub fn new() -> Self {
        Processor {
            marker: PhantomData,
        }
    }
}

impl<'a, A> System<'a> for Processor<A>
where
    A: Asset,
    A::Data: Into<Result<A>>,
{
    type SystemData = (
        FetchMut<'a, AssetStorage<A>>,
        Fetch<'a, Arc<ThreadPool>>,
        Fetch<'a, Time>,
        Option<Fetch<'a, HotReloadStrategy>>,
    );

    fn run(&mut self, (mut storage, pool, time, strategy): Self::SystemData) {
        use std::ops::Deref;

        storage.process(
            Into::into,
            time.frame_number(),
            &**pool,
            strategy.as_ref().map(Deref::deref),
        );
    }
}

/// A handle to an asset. This is usually what the
/// user deals with, the actual asset (`A`) is stored
/// in an `AssetStorage`.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Eq(bound = ""), Hash(bound = ""), PartialEq(bound = ""),
             Debug(bound = ""))]
pub struct Handle<A> {
    inner: HandleInner<A>,
}

impl<A> Handle<A> {
    /// Downgrades the handle and creates a `WeakHandle`.
    pub fn downgrade(&self) -> WeakHandle<A> {
        match self.inner {
            HandleInner::Alloc(ref a) => a.downgrade().into(),
            HandleInner::Library(ref a) => a.downgrade().into(),
        }
    }
}

impl<A> Component for Handle<A>
where
    A: Asset,
{
    type Storage = A::HandleStorage;
}

impl<A> From<HandleAlloc<A>> for Handle<A> {
    fn from(h: HandleAlloc<A>) -> Self {
        Handle {
            inner: HandleInner::Alloc(h),
        }
    }
}

impl<A> From<LibraryHandle<A>> for Handle<A> {
    fn from(h: LibraryHandle<A>) -> Self {
        Handle {
            inner: HandleInner::Library(h),
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Eq(bound = ""), Hash(bound = ""), PartialEq(bound = ""),
             Debug(bound = ""))]
pub struct HandleAlloc<A> {
    id: Arc<u32>,
    marker: PhantomData<A>,
}

impl<A> HandleAlloc<A> {
    /// Return the 32 bit id of this handle.
    pub fn id(&self) -> u32 {
        *self.id.as_ref()
    }

    /// Downgrades the handle and creates a `WeakHandle`.
    pub fn downgrade(&self) -> WeakAllocHandle<A> {
        let id = Arc::downgrade(&self.id);

        WeakAllocHandle {
            id,
            marker: PhantomData,
        }
    }

    /// Returns `true` if this is the only handle to the asset its pointing at.
    fn is_unique(&self) -> bool {
        Arc::strong_count(&self.id) == 1
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
enum HandleInner<A> {
    Alloc(HandleAlloc<A>),
    Library(LibraryHandle<A>),
}

impl<A> Clone for HandleInner<A> {
    fn clone(&self) -> Self {
        match *self {
            HandleInner::Alloc(ref a) => HandleInner::Alloc(a.clone()),
            HandleInner::Library(ref a) => HandleInner::Library(a.clone()),
        }
    }
}

impl<A> hash::Hash for HandleInner<A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match *self {
            HandleInner::Alloc(ref a) => {
                state.write_u8(0);
                hash::Hash::hash(a, state);
            }
            HandleInner::Library(ref a) => {
                state.write_u8(0);
                hash::Hash::hash(a, state);
            }
        }
    }
}

impl<A> PartialEq for HandleInner<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&HandleInner::Alloc(ref a), &HandleInner::Alloc(ref b)) => PartialEq::eq(a, b),
            (&HandleInner::Library(ref a), &HandleInner::Library(ref b)) => PartialEq::eq(a, b),
            _ => false,
        }
    }
}

pub(crate) enum Processed<A: Asset> {
    NewAsset {
        data: Result<FormatValue<A>>,
        handle: HandleAlloc<A>,
        name: String,
        tracker: Box<Tracker>,
    },
    HotReload {
        data: Result<FormatValue<A>>,
        handle: Handle<A>,
        name: String,
        old_reload: Box<Reload<A>>,
    },
}

/// A weak handle, which is useful if you don't directly need the asset
/// like in caches. This way, the asset can still get dropped (if you want that).
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct WeakAllocHandle<A> {
    id: Weak<u32>,
    marker: PhantomData<A>,
}

impl<A> WeakAllocHandle<A> {
    /// Tries to upgrade to a `Handle`.
    #[inline]
    pub fn upgrade(&self) -> Option<Handle<A>> {
        self.id
            .upgrade()
            .map(|id| {
                HandleAlloc {
                    id,
                    marker: PhantomData,
                }
            })
            .map(Into::into)
    }
}

pub struct WeakHandle<A> {
    inner: WeakInner<A>,
}

impl<A> WeakHandle<A> {
    pub fn upgrade(&self) -> Option<Handle<A>> {
        match self.inner {
            WeakInner::Alloc(ref a) => a.upgrade().map(Into::into),
            WeakInner::Library(ref a) => a.upgrade().map(Into::into),
        }
    }

    /// Returns `true` if the original handle is dead.
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.upgrade().is_none()
    }
}

impl<A> From<WeakAllocHandle<A>> for WeakHandle<A> {
    fn from(h: WeakAllocHandle<A>) -> Self {
        WeakHandle {
            inner: WeakInner::Alloc(h),
        }
    }
}

impl<A> From<WeakLibraryHandle<A>> for WeakHandle<A> {
    fn from(h: WeakLibraryHandle<A>) -> Self {
        WeakHandle {
            inner: WeakInner::Library(h),
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
enum WeakInner<A> {
    Alloc(WeakAllocHandle<A>),
    Library(WeakLibraryHandle<A>),
}

struct Library {
    id: Arc<u32>,
}

impl Library {
    pub fn handle<A, S>(&mut self, s: S) -> Handle<A>
    where
        A: Asset,
        S: Into<String>,
    {
        LibraryHandle {
            id: self.id.clone(),
            key: s.into(),
            marker: PhantomData,
        }.into()
    }
}

pub struct LibraryAllocator {
    alloc: Allocator,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct LibraryData<A> {
    // TODO: reconsider `String`
    map: FnvHashMap<String, HandleAlloc<A>>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Eq(bound = ""), Hash(bound = ""), PartialEq(bound = ""),
Debug(bound = ""))]
pub struct LibraryHandle<A> {
    id: Arc<u32>,
    key: String,
    marker: PhantomData<A>,
}

impl<A> LibraryHandle<A> {
    /// Downgrades the library handle.
    pub fn downgrade(&self) -> WeakLibraryHandle<A> {
        WeakLibraryHandle {
            id: Arc::downgrade(&self.id),
            key: self.key.clone(),
            marker: PhantomData,
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Debug(bound = ""))]
pub struct WeakLibraryHandle<A> {
    id: Weak<u32>,
    key: String,
    marker: PhantomData<A>,
}

impl<A> WeakLibraryHandle<A> {
    pub fn upgrade(&self) -> Option<LibraryHandle<A>> {
        self.id.upgrade().map(|id| {
            LibraryHandle {
                id,
                key: self.key.clone(),
                marker: PhantomData,
            }
        })
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct LibraryStorage<A> {
    bitset: BitSet,
    libs: VecStorage<LibraryData<A>>,
}

impl<A> LibraryStorage<A> {
    pub fn get(&self, h: &LibraryHandle<A>) -> Option<&HandleAlloc<A>> {
        let id = *h.id.as_ref();

        if self.bitset.contains(id) {
            unsafe { self.libs.get(id).map.get(&h.key) }
        } else {
            None
        }
    }
}
