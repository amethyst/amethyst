use std::fmt;
use std::marker::PhantomData;

use gfx_hal::{Backend, Device, MemoryType, MemoryTypeId};
use gfx_hal::memory::Requirements;
use memory::{Block, MemoryAllocator, Result};
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
    id: MemoryTypeId,
    allocations: usize,
    pd: PhantomData<fn() -> B>,
}

impl<B> RootAllocator<B> {
    pub fn new(id: MemoryTypeId) -> Self {
        RootAllocator {
            relevant: Relevant,
            id,
            allocations: 0,
            pd: PhantomData,
        }
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
        let memory = device.allocate_memory(self.id, reqs.size)?;
        let memory = Box::into_raw(Box::new(memory)); // Suboptimal
        self.allocations += 1;
        Ok(Block::new(memory, 0..reqs.size).set_tag(self.tag()))
    }

    fn free(&mut self, device: &B::Device, block: Block<B, Tag>) {
        assert_eq!(block.range().start, 0);
        device.free_memory(*unsafe { Box::from_raw(block.memory() as *const _ as *mut _) });
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
