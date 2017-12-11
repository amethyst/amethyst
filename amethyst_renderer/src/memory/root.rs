use std::fmt;
use std::marker::PhantomData;

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::memory::Requirements;
use memory::{calc_alignment_shift, Block, MemoryAllocator, MemorySubAllocator, Result};
use relevant::Relevant;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tag(*const ());

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemoryTag@{:p}", self.0)
    }
}

unsafe impl Send for Tag {}
unsafe impl Sync for Tag {}

#[derive(Debug)]
pub struct RootAllocator<B> {
    relevant: Relevant,
    memory_type: MemoryType,
    allocations: usize,
    pd: PhantomData<B>,
}

impl<B> RootAllocator<B> {
    pub fn new(memory_type: MemoryType) -> Self {
        RootAllocator {
            relevant: Relevant,
            memory_type,
            allocations: 0,
            pd: PhantomData,
        }
    }

    pub fn memory_type(&self) -> &MemoryType {
        &self.memory_type
    }

    pub fn tag(&mut self) -> Tag {
        Tag(self as *const _ as *const _)
    }
}

impl<B> MemoryAllocator<B> for RootAllocator<B>
where
    B: Backend,
{
    type Tag = Tag;
    type Info = ();

    fn alloc(&mut self, device: &B::Device, _: (), reqs: Requirements) -> Result<Block<B, Tag>> {
        assert_eq!(
            (1 << self.memory_type.id) & reqs.type_mask,
            1 << self.memory_type.id
        );
        let memory = device.allocate_memory(&self.memory_type, reqs.size)?;
        let memory = Box::into_raw(Box::new(memory)); // Unoptimal
        self.allocations += 1;
        Ok(Block::new(memory, 0..reqs.size).set_tag(self.tag()))
    }

    fn free(&mut self, device: &B::Device, block: Block<B, Tag>) {
        assert_eq!(block.offset, 0);
        device.free_memory(*unsafe { Box::from_raw(block.memory) });
        assert_eq!(unsafe { block.dispose() }, self.tag());
        self.allocations -= 1;
    }

    fn is_unused(&self) -> bool {
        self.allocations == 0
    }

    fn dispose(self, _: &B::Device) {
        assert!(self.is_unused());
        self.relevant.dispose()
    }
}
