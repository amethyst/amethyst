use gfx_hal::{Backend, MemoryType, MemoryTypeId};
use gfx_hal::memory::Requirements;

use memory::{Block, MemoryAllocator, MemorySubAllocator, Error};
use memory::arena::ArenaAllocator;
use memory::chunked::ChunkListAllocator;
use memory::root::RootAllocator;

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
    memory_type: MemoryType,
    root: RootAllocator<B>,
    arenas: ArenaAllocator<B, RootAllocator<B>>,
    chunks: ChunkListAllocator<B, RootAllocator<B>>,
}

impl<B> CombinedAllocator<B>
where
    B: Backend,
{
    pub fn new(
        memory_type: MemoryType,
        memory_type_id: MemoryTypeId,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        CombinedAllocator {
            memory_type,
            root: RootAllocator::new(memory_type_id),
            arenas: ArenaAllocator::new(arena_size, memory_type_id),
            chunks: ChunkListAllocator::new(chunk_size, min_chunk_size, memory_type_id),
        }
    }

    pub fn memory_type(&self) -> &MemoryType {
        &self.memory_type
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
    ) -> Result<Block<B, (Type, usize)>, Error> {
        match info {
            Type::Arena => self.arenas
                .alloc(&mut self.root, device, (), reqs)
                .map(|block| block.push_tag(Type::Arena)),
            Type::Chunk => self.chunks
                .alloc(&mut self.root, device, (), reqs)
                .map(|block| block.push_tag(Type::Chunk)),
        }
    }

    fn free(&mut self, device: &B::Device, block: Block<B, (Type, usize)>) {
        let (block, ty) = block.pop_tag();
        match ty {
            Type::Arena => self.arenas.free(&mut self.root, device, block),
            Type::Chunk => self.chunks.free(&mut self.root, device, block),
        }
    }

    fn is_unused(&self) -> bool {
        self.arenas.is_unused() && self.chunks.is_unused()
    }

    fn dispose(mut self, device: &B::Device) {
        self.arenas.dispose(&mut self.root, device);
        self.chunks.dispose(&mut self.root, device);
        self.root.dispose(device);
    }
}
