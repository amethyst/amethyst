

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::device::OutOfMemory;
use gfx_hal::memory::Requirements;

use allocator::{Allocator, Block, SubAllocator, calc_alignment_shift};
use allocator::memory::MemoryAllocator;
use allocator::arena::ArenaAllocator;
use allocator::chunked::ChunkListAllocator;

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
    memory: MemoryAllocator<B>,
    arenas: ArenaAllocator<B, MemoryAllocator<B>>,
    chunks: ChunkListAllocator<B, MemoryAllocator<B>>,
}

impl<B> CombinedAllocator<B>
where
    B: Backend,
{
    pub fn new(memory_type: MemoryType, arena_size: u64, chunk_size: u64, min_size: u64) -> Self {
        CombinedAllocator {
            memory: MemoryAllocator::new(memory_type),
            arenas: ArenaAllocator::new(arena_size, memory_type.id),
            chunks: ChunkListAllocator::new(chunk_size, min_size, memory_type.id),
        }
    }

    pub fn memory_type(&self) -> &MemoryType {
        self.memory.memory_type()
    }
}

impl<B> Allocator<B> for CombinedAllocator<B>
where
    B: Backend,
{
    type Info = Type;
    type Tag = (Type, usize);
    type Error = OutOfMemory;

    fn alloc(&mut self, device: &B::Device, info: Type, reqs: Requirements) -> Result<Block<B, (Type, usize)>, Self::Error> {
        match info {
            Type::Arena => {
                self.arenas.alloc(&mut self.memory, device, (), reqs).map(|block| block.push_tag(Type::Arena))
            }
            Type::Chunk => {
                self.chunks.alloc(&mut self.memory, device, (), reqs).map(|block| block.push_tag(Type::Chunk))
            }
        }
        
    }

    fn free(&mut self, device: &B::Device, block: Block<B, (Type, usize)>) {
        let (block, ty) = block.pop_tag();
        match ty {
            Type::Arena => {
                self.arenas.free(&mut self.memory, device, block)
            }
            Type::Chunk => {
                self.chunks.free(&mut self.memory, device, block)
            }
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
