use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam::sync::MsQueue;
use hibitset::BitSet;
use specs::{Component, Fetch, FetchMut, System, UnprotectedStorage, VecStorage};
use specs::common::Errors;

use BoxedErr;
use asset::Asset;
use error::AssetError;

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
    pub(crate) processed: Arc<MsQueue<Processed<A>>>,
    unused_handles: MsQueue<Handle<A>>,
}

impl<A: Asset> AssetStorage<A> {
    /// Creates a new asset storage.
    pub fn new() -> Self {
        AssetStorage {
            assets: Default::default(),
            bitset: Default::default(),
            handles: Default::default(),
            handle_alloc: Default::default(),
            //new_handles: MsQueue::new(),
            processed: Arc::new(MsQueue::new()),
            unused_handles: MsQueue::new(),
        }
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
    pub fn process<F>(&mut self, mut f: F, errors: &Errors)
    where
        F: FnMut(A::Data) -> Result<A, BoxedErr>,
    {
        while let Some(processed) = self.processed.try_pop() {
            let Processed {
                data,
                format,
                handle,
                name,
            } = processed;
            let assets = &mut self.assets;
            let bitset = &mut self.bitset;
            let handles = &mut self.handles;
            errors.execute::<AssetError, _>(|| {
                let asset = data.and_then(&mut f)
                    .map_err(|e| AssetError::new(name, format, e))?;

                let id = handle.id();
                bitset.add(id);
                handles.push(handle);

                // NOTE: the loader has to ensure that a handle will be used
                // together with a `Data` only once.
                unsafe {
                    assets.insert(id, asset);
                }

                Ok(())
            });
        }

        while let Some(i) = self.handles.iter().position(Handle::is_unused) {
            let old = self.handles.swap_remove(i);
            let id = i as u32;
            unsafe {
                self.assets.remove(id);
            }
            self.bitset.remove(id);
            self.unused_handles.push(old);
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
    type SystemData = (FetchMut<'a, AssetStorage<A>>, Fetch<'a, Errors>);

    fn run(&mut self, (mut storage, errors): Self::SystemData) {
        storage.process(Into::into, &errors);
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

    /// Returns `true` if this is the only handle to the asset its pointing at
    /// (excluding the handle owned by the asset storage).
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.id) == 2
    }

    fn is_unused(&self) -> bool {
        Arc::strong_count(&self.id) == 1
    }
}

impl<A> Component for Handle<A>
where
    A: Asset,
{
    type Storage = A::HandleStorage;
}

// TODO: may change with hot reloading
pub struct Processed<A: Asset> {
    pub data: Result<A::Data, BoxedErr>,
    pub format: String,
    pub handle: Handle<A>,
    pub name: String,
}
