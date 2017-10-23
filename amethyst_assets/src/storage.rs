use std::marker::PhantomData;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crossbeam::sync::MsQueue;
use hibitset::BitSet;
use rayon::ThreadPool;
use specs::{Component, Fetch, FetchMut, System, UnprotectedStorage, VecStorage};
use specs::common::Errors;

use BoxedErr;
use asset::{Asset, FormatValue};
use error::AssetError;
use reload::Reload;

/// An `Allocator`, holding a counter for producing unique IDs.
#[derive(Debug, Default)]
pub struct Allocator {
    store_count: AtomicUsize,
}

impl Allocator {
    /// Produces a new id.
    pub fn next_id(&self) -> usize {
        self.store_count.fetch_add(1, Ordering::Relaxed)
    }
}


/// An asset storage, storing the actual assets and allocating
/// handles to them.
pub struct AssetStorage<A: Asset> {
    assets: VecStorage<A>,
    bitset: BitSet,
    handles: Vec<Handle<A>>,
    handle_alloc: Allocator,
    last_reload: Instant,
    pub(crate) processed: Arc<MsQueue<Processed<A>>>,
    reloads: Vec<(WeakHandle<A>, Box<Reload<A>>)>,
    unused_handles: MsQueue<Handle<A>>,
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            assets: Default::default(),
            bitset: Default::default(),
            handles: Default::default(),
            handle_alloc: Default::default(),
            last_reload: Instant::now(),
            processed: Arc::new(MsQueue::new()),
            reloads: Vec::new(),
            unused_handles: MsQueue::new(),
        }
    }
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        Default::default()
    }

    /// Allocate a new handle.
    pub(crate) fn allocate(&self) -> Handle<A> {
        self.unused_handles
            .try_pop()
            .unwrap_or_else(|| self.allocate_new())
    }

    fn allocate_new(&self) -> Handle<A> {
        let id = self.handle_alloc.next_id() as u32;
        let handle = Handle {
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
        if let Some(asset) = self.get(handle).map(A::clone) {
            let h = self.allocate();

            let id = h.id();
            self.bitset.add(id);
            self.handles.push(h.clone());

            unsafe {
                self.assets.insert(id, asset);
            }

            Some(h)
        } else {
            None
        }
    }

    /// Get an asset from a given asset handle.
    pub fn get(&self, handle: &Handle<A>) -> Option<&A> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { self.assets.get(handle.id()) })
        } else {
            None
        }
    }

    /// Get an asset mutably from a given asset handle.
    pub fn get_mut(&mut self, handle: &Handle<A>) -> Option<&mut A> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { self.assets.get_mut(handle.id()) })
        } else {
            None
        }
    }

    /// Process finished asset data and maintain the storage.
    pub fn process<F>(&mut self, f: F, errors: &Errors, pool: &ThreadPool)
    where
        F: FnMut(A::Data) -> Result<A, BoxedErr>,
    {
        self.process_custom_drop(f, |_| {}, errors);
    }

    /// Process finished asset data and maintain the storage.
    /// This calls the `drop_fn` closure for assets that were removed from the storage.
    pub fn process_custom_drop<F, D>(&mut self, mut f: F, mut drop_fn: D, errors: &Errors)
    where
        D: FnMut(A),
        F: FnMut(A::Data) -> Result<A, BoxedErr>,
    {
        while let Some(processed) = self.processed.try_pop() {
            let assets = &mut self.assets;
            let bitset = &mut self.bitset;
            let handles = &mut self.handles;
            let reloads = &mut self.reloads;

            let f = &mut f;
            errors.execute::<AssetError, _>(move || {
                let (reload_obj, handle) = match processed {
                    Processed::NewAsset {
                        data,
                        format,
                        handle,
                        name,
                    } => {
                        let (asset, reload_obj) = data.map(
                            |FormatValue { data, reload }| (data, reload),
                        ).and_then(|(d, rel)| f(d).map(|a| (a, rel)))
                            .map_err(|e| AssetError::new(name, format, e))?;

                        let id = handle.id();
                        bitset.add(id);
                        handles.push(handle.clone());

                        // NOTE: the loader has to ensure that a handle will be used
                        // together with a `Data` only once.
                        unsafe {
                            assets.insert(id, asset);
                        }

                        (reload_obj, handle)
                    }
                    Processed::HotReload {
                        data,
                        format,
                        handle,
                        name,
                        old_reload,
                    } => {
                        let (asset, reload_obj) = match data.map(
                            |FormatValue { data, reload }| (data, reload),
                        ).and_then(|(d, rel)| f(d).map(|a| (a, rel)))
                            .map_err(|e| AssetError::new(name, format, e))
                        {
                            Ok(x) => x,
                            Err(e) => {
                                eprintln!("Failed to hot-reload: {}", e);

                                reloads.push((handle.downgrade(), old_reload));

                                return Ok(());
                            }
                        };

                        let id = handle.id();
                        assert!(bitset.contains(id));
                        unsafe {
                            let old = assets.get_mut(id);
                            *old = asset;
                        }

                        (reload_obj, handle)
                    }
                };

                // Add the reload obj if it is `Some`.
                if let Some(reload_obj) = reload_obj {
                    reloads.push((handle.downgrade(), reload_obj));
                }

                Ok(())
            });
        }

        let mut skip = 0;
        while let Some(i) = self.handles.iter().skip(skip).position(Handle::is_unique) {
            skip = i;
            let handle = self.handles.swap_remove(i);
            let id = handle.id();
            unsafe {
                drop_fn(self.assets.remove(id));
            }
            self.bitset.remove(id);

            // Can't reuse old handle here, because otherwise weak handles would still be valid.
            // TODO: maybe just store u32?
            self.unused_handles.push(Handle {
                id: Arc::new(id),
                marker: PhantomData,
            });
        }

        // Reload every seconds
        // TODO: more configuration
        let elapsed = self.last_reload.elapsed().as_secs();

        if elapsed >= 1 {
            self.last_reload = Instant::now();

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
                        let data = rel.reload();

                        let p = Processed::HotReload {
                            data,
                            name,
                            format,
                            handle,
                            old_reload,
                        };
                        processed.push(p);
                    });
                }
            }
        }
    }
}

impl<A: Asset> Drop for AssetStorage<A> {
    fn drop(&mut self) {
        let bitset = &self.bitset;
        unsafe { self.assets.clean(|id| bitset.contains(id)) }
    }
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
    A::Data: Into<Result<A, BoxedErr>>,
{
    type SystemData = (
        FetchMut<'a, AssetStorage<A>>,
        Fetch<'a, Arc<ThreadPool>>,
        Fetch<'a, Errors>,
    );

    fn run(&mut self, (mut storage, pool, errors): Self::SystemData) {
        storage.process(Into::into, &errors, &**pool);
    }
}

/// A handle to an asset. This is usually what the
/// user deals with, the actual asset (`A`) is stored
/// in an `AssetStorage`.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Eq(bound = ""), Hash(bound = ""), PartialEq(bound = ""),
             Debug(bound = ""))]
pub struct Handle<A: ?Sized> {
    id: Arc<u32>,
    marker: PhantomData<A>,
}

impl<A> Handle<A> {
    /// Return the 32 bit id of this handle.
    pub fn id(&self) -> u32 {
        *self.id.as_ref()
    }

    /// Downgrades the handle and creates a `WeakHandle`.
    pub fn downgrade(&self) -> WeakHandle<A> {
        let id = Arc::downgrade(&self.id);

        WeakHandle {
            id,
            marker: PhantomData,
        }
    }

    /// Returns `true` if this is the only handle to the asset its pointing at.
    fn is_unique(&self) -> bool {
        Arc::strong_count(&self.id) == 1
    }
}

impl<A> Component for Handle<A>
where
    A: Asset,
{
    type Storage = A::HandleStorage;
}

pub enum Processed<A: Asset> {
    NewAsset {
        data: Result<FormatValue<A>, BoxedErr>,
        format: &'static str,
        handle: Handle<A>,
        name: String,
    },
    HotReload {
        data: Result<FormatValue<A>, BoxedErr>,
        format: &'static str,
        handle: Handle<A>,
        name: String,
        old_reload: Box<Reload<A>>,
    },
}

/// A weak handle, which is useful if you don't directly need the asset
/// like in caches. This way, the asset can still get dropped (if you want that).
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct WeakHandle<A> {
    id: Weak<u32>,
    marker: PhantomData<A>,
}

impl<A> WeakHandle<A> {
    /// Tries to upgrade to a `Handle`.
    #[inline]
    pub fn upgrade(&self) -> Option<Handle<A>> {
        self.id.upgrade().map(|id| {
            Handle {
                id,
                marker: PhantomData,
            }
        })
    }

    /// Returns `true` if the original handle is dead.
    #[inline]
    pub fn is_dead(&self) -> bool {
        self.upgrade().is_none()
    }
}
