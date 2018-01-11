use gfx_hal::{Backend, MemoryType, MemoryTypeId, MemoryProperties};
use gfx_hal::memory::{Properties, Requirements};

use memory::{Block, Error, ErrorKind, MemoryAllocator, Result};
use memory::combined::{CombinedAllocator, Type};

struct Heap {
    size: u64,
    used: u64,
}

impl Heap {
    fn available(&self) -> u64 {
        self.size - self.used
    }

    fn alloc(&mut self, size: u64) {
        self.used += size;
    }

    fn free(&mut self, size: u64) {
        self.used -= size;
    }
}

pub struct SmartAllocator<B: Backend> {
    allocators: Vec<CombinedAllocator<B>>,
    heaps: Vec<Heap>,
}

impl<B> SmartAllocator<B>
where
    B: Backend,
{
    pub fn new(
        memory_properties: MemoryProperties,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        SmartAllocator {
            allocators: memory_properties.memory_types
                .into_iter()
                .enumerate()
                .map(|(index, memory_type)| {
                    CombinedAllocator::new(memory_type, MemoryTypeId(index), arena_size, chunk_size, min_chunk_size)
                })
                .collect(),
            heaps: memory_properties.memory_heaps.into_iter().map(|size| Heap { size, used: 0 }).collect(),
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
    
        let ref mut heaps = self.heaps;
        let allocators = self.allocators
            .iter_mut()
            .enumerate();

        let mut compatible_count = 0;
        let (index, allocator) = allocators.filter(|&(index, ref allocator)| {
            let memory_type = allocator.memory_type();
            ((1 << index) & reqs.type_mask) == (1 << index) && memory_type.properties.contains(prop)
        }).filter(|&(_, ref allocator)| {
            compatible_count += 1;
            heaps[allocator.memory_type().heap_index].available() >= (reqs.size + reqs.alignment)
        }).next().ok_or(Error::from(
            if compatible_count == 0 {
                ErrorKind::NoCompatibleMemoryType
            } else {
                ErrorKind::OutOfMemory
            }
        ))?;

        let block = allocator.alloc(device, ty, reqs)?;
        heaps[allocator.memory_type().heap_index].alloc(block.size());

        Ok(block.push_tag(index))
    }

    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>) {
        let (block, index) = block.pop_tag();
        self.heaps[self.allocators[index].memory_type().heap_index].free(block.size());
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
