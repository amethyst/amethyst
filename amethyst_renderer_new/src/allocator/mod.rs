
use std::cmp::Eq;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::{Add, Range, Rem, Sub};

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::memory::{Properties, Requirements};
use relevant::Relevant;

mod arena;
mod chunked;
mod combined;
mod memory;
mod smart;


pub use self::smart::SmartAllocator;
pub use self::combined::Type as AllocationType;

/// Tagged block of memory.
/// It is relevant type and can't be silently dropped.
/// User must return this block to the same `Allocator` it came from.
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
        A: Allocator<B, Tag = T>,
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

    /// Helper merthod to check if `other` block is sub-block of `self`
    pub fn contains<Y>(&self, other: &Block<B, Y>) -> bool {
        self.memory == other.memory && self.offset <= other.offset &&
            self.offset + self.size >= other.offset + other.size
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
            tag,
            offset,
            size,
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
    /// memory was freed
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

pub trait Allocator<B: Backend> {
    type Info;
    type Tag: Debug + Copy + Send + Sync;
    type Error: Error;

    fn alloc(
        &mut self,
        device: &B::Device,
        info: Self::Info,
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>, Self::Error>;
    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, device: &B::Device);
}

pub trait SubAllocator<B: Backend> {
    type Owner;
    type Info;
    type Tag: Debug + Copy + Send + Sync;
    type Error: Error;

    fn alloc(
        &mut self,
        owner: &mut Self::Owner,
        device: &B::Device,
        info: Self::Info,
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>, Self::Error>;
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
