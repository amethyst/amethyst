use gfx_hal::{Backend, MemoryType};
use gfx_hal::memory::{Properties, Requirements};

use memory::{Block, Error, ErrorKind, MemoryAllocator, Result};
use memory::combined::{CombinedAllocator, Type};

pub struct SmartAllocator<B: Backend> {
    allocators: Vec<CombinedAllocator<B>>,
}

impl<B> SmartAllocator<B>
where
    B: Backend,
{
    pub fn new(
        memory_types: Vec<MemoryType>,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        SmartAllocator {
            allocators: memory_types
                .into_iter()
                .map(|mt| {
                    CombinedAllocator::new(mt, arena_size, chunk_size, min_chunk_size)
                })
                .collect(),
        }
    }
}

impl<B> MemoryAllocator<B> for SmartAllocator<B>
where
    B: Backend,
{
    type Info = (Type, Properties);
    type Tag = (usize, (Type, usize));

    fn alloc(
        &mut self,
        device: &B::Device,
        (ty, prop): (Type, Properties),
        reqs: Requirements,
    ) -> Result<Block<B, Self::Tag>> {
        let (index, allocator) = self.allocators
            .iter_mut()
            .enumerate()
            .find(|&(_, ref allocator)| {
                let memory_type = allocator.memory_type();
                ((1 << memory_type.id) & reqs.type_mask) == (1 << memory_type.id)
                    && memory_type.properties.contains(prop)
            })
            .ok_or(Error::from(ErrorKind::NoCompatibleMemoryType))?;
        let block = allocator.alloc(device, ty, reqs)?;
        Ok(block.push_tag(index))
    }

    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>) {
        let (block, index) = block.pop_tag();
        self.allocators[index].free(device, block);
    }

    fn is_unused(&self) -> bool {
        self.allocators.iter().all(MemoryAllocator::is_unused)
    }

    fn dispose(mut self, device: &B::Device) {
        self.allocators.drain(..).for_each(|allocator| {
            allocator.dispose(device);
        });
    }
}
