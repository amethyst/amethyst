//!
//! Memory management for rendering.
//! 

use std::cmp::Eq;
use std::fmt::Debug;
use std::ops::{Add, Range, Rem, Sub};

use gfx_hal::Backend;
use gfx_hal::buffer::Usage as BufferUsage;
use gfx_hal::image::Usage as ImageUsage;
use gfx_hal::memory::{Pod, Properties, Requirements};
use relevant::Relevant;

mod arena;
mod chunked;
mod combined;
mod allocator;
mod root;
mod smart;

pub use self::allocator::{Allocator, Buffer, Image, WeakBuffer, WeakImage};
pub use self::smart::SmartAllocator;


error_chain! {
    foreign_links {
        BindError(::gfx_hal::device::BindError);
        // ViewError(::gfx_hal::buffer::ViewError);
        BufferCreationError(::gfx_hal::buffer::CreationError);
        ImageCreationError(::gfx_hal::image::CreationError);
        OutOfMemory(::gfx_hal::device::OutOfMemory);
    }

    errors {
        NoCompatibleMemoryType {}
        BufferUsageAndProperties(usage: BufferUsage, properties: Properties) {
            description("No memory type supports both usage and properties specified")
            display("No memory type supports both
                usage ({:?}) and properties ({:?}) specified", usage, properties)
        }
        ImageUsageAndProperties(usage: ImageUsage, properties: Properties) {
            description("No memory type supports both usage and properties specified")
            display("No memory type supports both
                usage ({:?}) and properties ({:?}) specified", usage, properties)
        }
    }
}


/// Tagged block of memory.
/// It is relevant type and can't be silently dropped.
/// User must return this block to the same `MemoryAllocator` it came from.
#[derive(Debug)]
pub struct Block<B: Backend, T> {
    relevant: Relevant,
    tag: T,
    memory: *mut B::Memory,
    offset: u64,
    size: u64,
}


impl<B> Block<B, ()>
where
    B: Backend,
{
    pub fn new(memory: *mut B::Memory, range: Range<u64>) -> Self {
        assert!(range.start <= range.end);
        Block {
            relevant: Relevant,
            tag: (),
            memory,
            offset: range.start,
            size: range.end - range.start,
        }
    }
}


impl<B, T> Block<B, T>
where
    B: Backend,
{
    /// Free this block returning it to the origin
    pub fn free<A>(self, origin: &mut A, device: &B::Device)
    where
        A: MemoryAllocator<B, Tag = T>,
        T: Debug + Copy + Send + Sync,
    {
        origin.free(device, self);
    }

    pub fn memory(&self) -> &B::Memory {
        // Has to be valid
        unsafe { &*self.memory }
    }

    pub fn range(&self) -> Range<u64> {
        self.offset..self.size + self.offset
    }

    /// Helper method to check if `other` block is sub-block of `self`
    pub fn contains<Y>(&self, other: &Block<B, Y>) -> bool {
        self.memory == other.memory && self.offset <= other.offset
            && self.offset + self.size >= other.offset + other.size
    }

    /// Push additional tag value to this block.
    /// Tags form a stack - e.g. LIFO
    pub fn push_tag<Y>(self, value: Y) -> Block<B, (Y, T)> {
        let Block {
            relevant,
            memory,
            tag,
            offset,
            size,
        } = self;
        Block {
            relevant,
            memory,
            tag: (value, tag),
            offset,
            size,
        }
    }

    /// Replace tag attached to this block
    pub fn replace_tag<Y>(self, value: Y) -> (Block<B, Y>, T) {
        let Block {
            relevant,
            memory,
            tag,
            offset,
            size,
        } = self;
        (
            Block {
                relevant,
                memory,
                tag: value,
                offset,
                size,
            },
            tag,
        )
    }

    /// Set tag to this block.
    /// Drops old tag.
    pub fn set_tag<Y>(self, value: Y) -> Block<B, Y> {
        let Block {
            relevant,
            memory,
            offset,
            size,
            ..
        } = self;
        Block {
            relevant,
            memory,
            tag: value,
            offset,
            size,
        }
    }

    /// Dispose of this block.
    /// Returns tag value.
    /// This is unsafe as the caller must ensure that
    /// the memory of the block won't be used.
    /// Typically by dropping resource (`Buffer` or `Image`) that occupy this memory.
    pub unsafe fn dispose(self) -> T {
        self.relevant.dispose();
        self.tag
    }
}

impl<B, T, Y> Block<B, (Y, T)>
where
    B: Backend,
{
    /// Pop top tag value from this block
    /// Tags form a stack - e.g. LIFO
    fn pop_tag(self) -> (Block<B, T>, Y) {
        let Block { .. } = self;
        let Block {
            relevant,
            memory,
            tag: (value, tag),
            offset,
            size,
        } = self;
        (
            Block {
                relevant,
                memory,
                tag,
                offset,
                size,
            },
            value,
        )
    }
}

pub trait MemoryAllocator<B: Backend> {
    type Info;
    type Tag: Debug + Copy + Send + Sync;

    fn alloc(
        &mut self,
        device: &B::Device,
        info: Self::Info,
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>>;
    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, device: &B::Device);
}

pub trait MemorySubAllocator<B: Backend> {
    type Owner;
    type Info;
    type Tag: Debug + Copy + Send + Sync;

    fn alloc(
        &mut self,
        owner: &mut Self::Owner,
        device: &B::Device,
        info: Self::Info,
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>>;
    fn free(&mut self, owner: &mut Self::Owner, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, owner: &mut Self::Owner, device: &B::Device);
}


pub fn calc_alignment_shift<T>(alignment: T, offset: T) -> T
where
    T: From<u8> + Add<Output = T> + Sub<Output = T> + Rem<Output = T> + Eq + Copy,
{
    if offset == 0.into() {
        0.into()
    } else {
        alignment - (offset - 1.into()) % alignment - 1.into()
    }
}


pub fn shift_for_alignment<T>(alignment: T, offset: T) -> T
where
    T: From<u8> + Add<Output = T> + Sub<Output = T> + Rem<Output = T> + Eq + Copy,
{
    offset + calc_alignment_shift(alignment, offset)
}


/// Cast `Vec` of one `Pod` type into `Vec` of another `Pod` type
/// Align and size of input type must be divisible by align and size of output type
/// Converting from arbitrary `T: Pod` into `u8` is always possible
/// as `u8` has size and align equal to 1
pub fn cast_pod_vec<T, Y>(mut vec: Vec<T>) -> Vec<Y>
where
    T: Pod,
    Y: Pod,
{
    use std::mem::{align_of, forget, size_of};

    debug_assert_eq!(align_of::<T>() % align_of::<Y>(), 0);
    debug_assert_eq!(size_of::<T>() % size_of::<Y>(), 0);

    let tsize = size_of::<T>();
    let ysize = size_of::<Y>();

    let p = vec.as_mut_ptr();
    let s = vec.len();
    let c = vec.capacity();

    unsafe {
        forget(vec);
        Vec::from_raw_parts(p as *mut Y, (s * tsize) / ysize, (c * tsize) / ysize)
    }
}
