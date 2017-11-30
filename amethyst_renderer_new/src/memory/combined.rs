

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::device::OutOfMemory;
use gfx_hal::memory::Requirements;

use memory::{MemoryAllocator, Block, Result, MemorySubAllocator, calc_alignment_shift};
use memory::root::RootAllocator;
use memory::arena::ArenaAllocator;
use memory::chunked::ChunkListAllocator;

#[derive(Clone, Copy, Debug)]
pub enum Type {
    Arena,
    Chunk,
}

#[derive(Debug)]
pub struct CombinedAllocator<B>
where
    B: Backend,
{
    memory: RootAllocator<B>,
    arenas: ArenaAllocator<B, RootAllocator<B>>,
    chunks: ChunkListAllocator<B, RootAllocator<B>>,
}

impl<B> CombinedAllocator<B>
where
    B: Backend,
{
    pub fn new(
        memory_type: MemoryType,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        CombinedAllocator {
            memory: RootAllocator::new(memory_type),
            arenas: ArenaAllocator::new(arena_size, memory_type.id),
            chunks: ChunkListAllocator::new(chunk_size, min_chunk_size, memory_type.id),
        }
    }

    pub fn memory_type(&self) -> &MemoryType {
        self.memory.memory_type()
    }
}

impl<B> MemoryAllocator<B> for CombinedAllocator<B>
where
    B: Backend,
{
    type Info = Type;
    type Tag = (Type, usize);

    fn alloc(
        &mut self,
        device: &B::Device,
        info: Type,
        reqs: Requirements,
    ) -> Result<Block<B, (Type, usize)>> {
        match info {
            Type::Arena => {
                self.arenas.alloc(&mut self.memory, device, (), reqs).map(
                    |block| block.push_tag(Type::Arena),
                )
            }
            Type::Chunk => {
                self.chunks.alloc(&mut self.memory, device, (), reqs).map(
                    |block| block.push_tag(Type::Chunk),
                )
            }
        }

    }

    fn free(&mut self, device: &B::Device, block: Block<B, (Type, usize)>) {
        let (block, ty) = block.pop_tag();
        match ty {
            Type::Arena => self.arenas.free(&mut self.memory, device, block),
            Type::Chunk => self.chunks.free(&mut self.memory, device, block),
        }
    }

    fn is_unused(&self) -> bool {
        self.arenas.is_unused() && self.chunks.is_unused()
    }

    fn dispose(mut self, device: &B::Device) {
        self.arenas.dispose(&mut self.memory, device);
        self.chunks.dispose(&mut self.memory, device);
        self.memory.dispose(device);
    }
}
