use std::fmt::Debug;
use std::ops::Range;

use gfx_hal::Backend;


use memory::MemoryAllocator;
use relevant::Relevant;


/// Tagged block of memory.
/// It is relevant type and can't be silently dropped.
/// User must return this block to the same `MemoryAllocator` it came from.
#[derive(Debug)]
pub struct Block<B: Backend, T> {
    relevant: Relevant,
    tag: T,
    memory: *const B::Memory,
    range: Range<u64>,
}


impl<B> Block<B, ()>
where
    B: Backend,
{
    pub fn new(memory: *const B::Memory, range: Range<u64>) -> Self {
        assert!(range.start <= range.end);
        Block {
            relevant: Relevant,
            tag: (),
            memory,
            range,
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
        self.range.clone()
    }

    pub fn size(&self) -> u64 {
        self.range.end - self.range.start
    }

    /// Helper method to check if `other` block is sub-block of `self`
    pub fn contains<Y>(&self, other: &Block<B, Y>) -> bool {
        self.memory == other.memory && self.range.start <= other.range.start
            && self.range.end >= other.range.end
    }

    /// Push additional tag value to this block.
    /// Tags form a stack - e.g. LIFO
    pub fn push_tag<Y>(self, value: Y) -> Block<B, (Y, T)> {
        let Block {
            relevant,
            memory,
            tag,
            range,
        } = self;
        Block {
            relevant,
            memory,
            tag: (value, tag),
            range,
        }
    }

    /// Replace tag attached to this block
    pub fn replace_tag<Y>(self, value: Y) -> (Block<B, Y>, T) {
        let Block {
            relevant,
            memory,
            tag,
            range,
        } = self;
        (
            Block {
                relevant,
                memory,
                tag: value,
                range,
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
            range,
            ..
        } = self;
        Block {
            relevant,
            memory,
            tag: value,
            range,
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
    pub fn pop_tag(self) -> (Block<B, T>, Y) {
        let Block { .. } = self;
        let Block {
            relevant,
            memory,
            tag: (value, tag),
            range,
        } = self;
        (
            Block {
                relevant,
                memory,
                tag,
                range,
            },
            value,
        )
    }
}

