

use std::cmp::{Ordering, PartialOrd, max, min};
use std::collections::VecDeque;
use std::ops::{Add, Deref, DerefMut};
use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::memory::Properties;
use gfx_hal::buffer::Usage as BufferUsage;
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::device::OutOfMemory;
use gfx_hal::mapping::Error as MappingError;

use specs::{Fetch, FetchMut};

use memory::{Item, MemoryError, MemoryErrorKind, SmartAllocator, create_buffer, create_image, destroy_buffer, destroy_image};
use relevant::Relevant;


error_chain!{
    links {
        Memory(MemoryError, MemoryErrorKind);
    }
}


type EpochData<'a> = Fetch<'a, CurrentEpoch>;
type EpochDataMut<'a> = FetchMut<'a, CurrentEpoch>;

/// Epoch identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);

impl Epoch {
    pub fn new() -> Self {
        Epoch(0)
    }
}

impl Add<u64> for Epoch {
    type Output = Epoch;
    fn add(self, add: u64) -> Epoch {
        Epoch(self.0 + add)
    }
}

/// Epoch counter.
/// Place it somewhere where all `Ec` users can access it
#[derive(Debug)]
pub struct CurrentEpoch(u64);

impl CurrentEpoch {
    pub fn new() -> Self {
        CurrentEpoch(1)
    }

    pub fn now(&self) -> Epoch {
        Epoch(self.0)
    }

    pub fn advance(&mut self) {
        self.0 += 1;
    }
}

/// Weak epoch pointer to `T`.
/// It will expire after some `Epoch`.
pub struct Ec<T> {
    ptr: *const T,
    valid_through: u64,
}

impl<T> Ec<T> {
    /// Get `Epoch` after which this `Ec` will expire.
    pub fn valid_through(&self) -> Epoch {
        Epoch(self.valid_through)
    }

    /// Get reference to the pointer value.
    /// Returns `Some` if `Ec` hasn't expired yet
    /// (CurrentEpoch is less than `self.valid_through()`).
    /// Returns `None` otherwise.
    #[inline]
    pub fn get<'a>(&'a self, current: &CurrentEpoch) -> Option<&'a T> {
        if self.valid_through <= current.0 {
            unsafe { Some(&*self.ptr) }
        } else {
            None
        }
    }
}

/// Strong epoch pointer to `T`.
/// It will hold value alive and can't be dropped until `CurrentEpoch`
/// is equal to last `Epoch` spcified in `make_valid_through` and `borrow`
pub struct Eh<T> {
    relevant: Relevant,
    ptr: *const T,
    valid_through: u64,
}

impl<T> From<Box<T>> for Eh<T> {
    fn from(b: Box<T>) -> Self {
        Eh {
            relevant: Relevant,
            ptr: Box::into_raw(b),
            valid_through: 0,
        }
    }
}

impl<T> Eh<T> {
    /// Wrap value into `Eh`
    pub fn new(value: T) -> Self {
        Eh {
            relevant: Relevant,
            ptr: Box::into_raw(Box::new(value)),
            valid_through: 0,
        }
    }

    /// Make all new `Ec` from this to be valid
    /// until specifyed `Epoch` expired
    pub fn make_valid_through(this: &mut Self, epoch: Epoch) {
        this.valid_through = max(this.valid_through, epoch.0);
    }

    /// Get last epoch for which `Eh` whould be valid
    pub fn valid_through(this: &Self) -> Epoch {
        Epoch(this.valid_through)
    }

    /// Borrow `Ec` from this `Eh`
    /// `Ec` will expire after specified `Epoch`
    pub fn borrow(this: &mut Self, epoch: Epoch) -> Ec<T> {
        Self::make_valid_through(this, epoch);
        Ec {
            ptr: this.ptr,
            valid_through: this.valid_through,
        }
    }

    ///
    pub fn dispose(self, current: &CurrentEpoch) {
        assert!(self.valid_through < current.0);
        self.relevant.dispose()
    }
}

impl<T> Deref for Eh<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

pub struct Epochal<T> {
    item: T,
    valid_through: Epoch,
}

impl<T> Epochal<T> {
    /// Make all new `Ec` borrowed this to be valid
    /// until specifyed `Epoch` expired
    pub fn make_valid_through(this: &mut Self, epoch: Epoch) {
        this.valid_through = max(this.valid_through, epoch);
    }

    /// Convert `Epochal` into `Eh` so that user can get shared
    /// reference to it as `Ec`
    pub fn into_shared(this: Self) -> Eh<T> {
        let mut eh = Eh::new(this.item);
        Eh::make_valid_through(&mut eh, this.valid_through);
        eh
    }

    fn is_valid_for(this: &Self, epoch: Epoch) -> bool {
        this.valid_through >= epoch
    }
}

impl<T> Deref for Epochal<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.item
    }
}

impl<T> DerefMut for Epochal<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

struct EpochalDeletionQueue<T> {
    offset: u64,
    queue: VecDeque<Vec<Epochal<T>>>,
    clean_vecs: Vec<Vec<Epochal<T>>>,
}

impl<T> EpochalDeletionQueue<T> {
    fn new() -> Self {
        EpochalDeletionQueue {
            offset: 0,
            queue: VecDeque::new(),
            clean_vecs: Vec::new(),
        }
    }

    fn add(&mut self, epochal: Epochal<T>) {
        let index = (epochal.valid_through.0 - self.offset) as usize;
        let ref mut queue = self.queue;
        let ref mut clean_vecs = self.clean_vecs;

        let len = queue.len();
        queue.extend((len..index).map(|_| {
            clean_vecs.pop().unwrap_or_else(|| Vec::new())
        }));
        queue[index].push(epochal);
    }

    fn clean<F>(&mut self, current: &CurrentEpoch, mut f: F)
    where
        F: FnMut(T),
    {
        let index = (current.now().0 - self.offset) as usize;
        let len = self.queue.len();

        for mut vec in self.queue.drain(..min(index, len)) {
            for epochal in vec.drain(..) {
                assert!(!Epochal::is_valid_for(&epochal, current.now()));
                f(epochal.item);
            }
            self.clean_vecs.push(vec);
        }
        self.offset += index as u64;
    }
}

pub type Buffer<B: Backend> = Epochal<Item<B, B::Buffer>>;
pub type Image<B: Backend> = Epochal<Item<B, B::Image>>;

pub type SharedBuffer<B: Backend> = Ec<Item<B, B::Buffer>>;
pub type SharedImage<B: Backend> = Ec<Item<B, B::Image>>;

pub struct EpochalManager<B: Backend> {
    allocator: SmartAllocator<B>,
    buffer_deletion_queue: EpochalDeletionQueue<Item<B, B::Buffer>>,
    image_deletion_queue: EpochalDeletionQueue<Item<B, B::Image>>,
}

impl<B> EpochalManager<B>
where
    B: Backend,
{
    pub fn new(
        memory_types: Vec<MemoryType>,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        EpochalManager {
            allocator: SmartAllocator::new(memory_types, arena_size, chunk_size, min_chunk_size),
            buffer_deletion_queue: EpochalDeletionQueue::new(),
            image_deletion_queue: EpochalDeletionQueue::new(),
        }
    }

    pub fn create_buffer(
        &mut self,
        device: &B::Device,
        size: u64,
        stride: u64,
        usage: BufferUsage,
        properties: Properties,
        transient: bool,
    ) -> Result<Buffer<B>> {
        let buffer = create_buffer(
            &mut self.allocator,
            device,
            size,
            stride,
            usage,
            properties,
            transient,
        )?;
        Ok(Epochal {
            item: buffer,
            valid_through: Epoch::new(),
        })
    }

    pub fn create_image(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
        properties: Properties,
    ) -> Result<Image<B>> {
        let image = create_image(
            &mut self.allocator,
            device,
            kind,
            level,
            format,
            usage,
            properties,
        )?;
        Ok(Epochal {
            item: image,
            valid_through: Epoch::new(),
        })
    }

    pub fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.buffer_deletion_queue.add(buffer);
    }

    pub fn destroy_image(&mut self, image: Image<B>) {
        self.image_deletion_queue.add(image);
    }

    pub fn cleanup(&mut self, device: &B::Device, current: &CurrentEpoch) {
        let ref mut allocator = self.allocator;
        self.image_deletion_queue.clean(
            current,
            |image| { destroy_image(allocator, device, image); },
        );
        self.buffer_deletion_queue.clean(current, |buffer| {
            destroy_buffer(allocator, device, buffer);
        });
    }
}
