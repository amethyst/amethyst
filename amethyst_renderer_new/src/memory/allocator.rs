
use std::mem::{drop, forget};
use std::ptr::read;
use std::result::Result as StdResult;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::buffer::{Usage as BufferUsage, complete_requirements};
use gfx_hal::device::OutOfMemory;
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::memory::{Properties, Requirements};
use relevant::Relevant;

pub struct Block<B: Backend, T> {
    memory: *mut B::Memory,
    tag: T,
    offset: usize,
    size: usize,
}

impl<B, T> Relevant for Block<B, T>
where
    B: Backend,
{}

impl<B, T> Block<B, T>
where
    B: Backend,
{
    fn dispose<A>(self, owner: &mut A)
    where
        A: Allocator<B, Tag=T>,
    {
        assert_eq!(owner as *const _, self.owner as *const _);
        owner.free(self);
    }

    fn contains<Y>(&self, other: &Block<B, Y>) -> bool {
        self.memory == other.memory &&
        self.offset <= other.offset &&
        self.offset + self.size >= other.offset + other.size
    }

    fn convert<Y, F>(mut self, f: F) -> Block<B, Y>
    where
        F: FnOnce(T) -> Y,
    {
        let converted = Block {
            memory: block.memory,
            offset: block.offset,
            size: block.size,
            tag: f(block.tag),
        };
        forget(block);
        converted
    }
}

trait Allocator<B: Backend>: Relevant {
    type Info;
    type Tag: Debug + Copy + Send + Sync;
    type Error: Error;
    
    fn alloc(&mut self, device: &B::Device, info: Self::Info, reqs: Requirements) -> Result<Block<B, Self::Tag>, Self::Error>;
    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, device: &B::Device);
}

trait SubAllocator<B: Backend>: Relevant {
    type Owner;
    type Info;
    type Tag: Debug + Copy + Send + Sync;
    type Error: Error;
    
    fn alloc(&mut self, owner: &mut Self::Owner, device: &B::Device, info: Self::Info, reqs: Requirements) -> Result<Block<B, Self::Tag>, Self::Error>;
    fn free(&mut self, owner: &mut Self::Owner, device: &B::Device, block: Block<B, Self::Tag>);
    fn is_unused(&self) -> bool;
    fn dispose(self, owner: &mut Self::Owner, device: &B::Device);
}

impl<'a, B> SubAllocator<'a, B> for A
where
    B: Backend,
    A: Allocator<B>,
{
    type Owner = ();
    type Info = A::Info;
    type Tag = A::Tag;
    type Error = A::Error;
    fn align(&self, info: Self::Info, reqs: Requirements) -> usize {
        Allocator::align(self, info, reqs)
    }
    fn alloc(&mut self, _: &mut (), device: &B::Device, info: Self::Info, reqs: Requirements) -> Result<Block<B, Self::Tag>, A::Error> {
        Allocator::alloc(self, device, info, reqs)
    }
    fn free(&mut self, _: &mut (), device: &B::Device, info: Self::Info, block: Block<B, Self::Tag>) {
        Allocator::free(self, device, block);
    }
    fn is_unused(&self) -> bool {
        Allocator::is_unused(self)
    }
    fn dispose(self, _: &mut (), device: &B::Device) {
        Allocator::dispose(self, device);
    }
}



/// Memory node for transient allocations
struct ArenaNode<B: Backend, A: Allocator> {
    block: Block<B, A::Tag>,
    used: usize,
    freed: usize,
}

impl<B, A> Relevant for ArenaNode<B, A>
where
    B: Backend,
    A: Allocator<B>,
{}

impl<B, A> ArenaNode<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    fn new(block: Block<B, A::Tag>) -> Self {
        ArenaNode {
            block,
            used: 0,
            freed: 0,
        }
    }
}

impl<B> SubAllocator<B> for ArenaNode<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    type Owner = A;
    type Info = ();
    type Tag = ();
    type Error = OutOfMemory;

    
    fn alloc(&mut self, _: &mut A, _: &B::Device, _: (), reqs: Requirements) -> Result<Block<B, Self::Tag>, OutOfMemory> {

        let offset = self.block.offset + self.used;
        let total_size = reqs.size + calc_alignment_shift(reqs.alignment, offset);

        if self.block.size - self.used < total_size {
            Err(OutOfMemory)
        } else {
            self.used += total_size;
            Ok(Block { tag: (), memory: self.block.memory, offset, total_size })
        }
    }
    fn free(&mut self, _: &mut A, _: &B::Device, block: Block<B, Self::Tag>) {
        assert!(self.block.contains(block));
        self.freed += block.size;
        forget(block);
    }
    fn is_unused(&self) -> bool {
        self.freed == self.used
    }
    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert_eq!(self.is_unused());
        unsafe {
            owner.free(device, read(&mut self.block));
            forget(self);
        }
    }
}





struct ArenaAllocator<B: Backend, A: Allocator<B>> {
    id: usize,
    arena_size: usize,
    hot: Option<Box<ArenaNode<B, A>>>,
    freed: usize,
    nodes: VecDeque<Box<ArenaNode<B, A>>>,
}

impl<B, A> ArenaAllocator<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    fn new(arena_size: usize, id: usize) -> Self {
        ArenaAllocator {
            id,
            arena_size,
            hot: None,
            freed: 0,
            nodes: VecDeque::new(),
        }
    }

    fn cleanup(&mut self, owner: &mut A, device: &B::Device) {
        while self.nodes.front().map(|node| node.is_unused()).unwrap_or(false) {
            self.nodes.pop_front().dispose(owner, device);
            self.freed += 1;
        }
    }

    fn allocate_node(&mut self, owner: &mut A, device: &B::Device, reqs: Requirements) -> Result<ArenaNode<B, A>, A::Error> {
        let arena_size = ((reqs.size - 1) / self.arena_size + 1) * self.arena_size;
        let arena_requirements = Requirements {
            type_mask: 1 << self.id,
            size: arena_size,
            alignment: reqs.alignment
        };
        let arena_block = owner.alloc(device, info, arena_requirements)?;
        ArenaNode::new(arena_block)
    }
}

impl<B, A> Relevant for ArenaAllocator<B, A> where B: Backend, A: Allocator<B>, {}

impl<B, A> SubAllocator<B> for ArenaAllocator<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    type Owner = A;
    type Info = A::Info;
    type Tag = usize;
    type Error = A::Error;

    fn alloc(&mut self, owner: &mut A, device: &B::Device, info: A::Info, reqs: Requirements) -> Result<Block<B, Self::Tag>, A::Error> {
        assert!((1 << self.id) & reqs.type_mask);
        let count = self.nodes.len();
        if let Some(ref mut hot) = self.hot.as_mut() {
            match hot.alloc(device, (), reqs) {
                Ok(block) => return Ok(block.convert(|()| count)),
                Err(OutOfMemory) => {}
            }
        };

        if let Some(hot) = self.hot {
            self.nodes.push(hot);
        }
        
        let node = self.allocate_node(owner, device, reqs);
        let block = node.alloc(device, (), reqs).unwrap();
        hot.take().map(|hot| self.nodes.push(hot));
        hot = Some(Box::new(node));
        let count = self.nodes.len();
        Ok(block.convert(|()| count))
    }
    fn free(&mut self, owner: &mut A, device: &B::Device, block: Block<B, Self::Tag>) {
        let index = block.tag - self.freed;
        let block = block.convert(|_| ());

        match self.nodes.len() {
            len if len == index {
                self.hot.free(device, block);
            }
            len if len > index {
                self.nodes[index].free(device, block);
                self.cleanup(owner, device);
            }
            _ => {
                unreachable!()
            }
        }
    }
    fn is_unused(&self) -> bool {
        self.nodes.is_empty() && self.hot.is_unused()
    }

    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert_eq!(self.is_unused());
        unsafe {
            self.hot.take().dispose(owner, device);
            read(&mut self.nodes);
            forget(self);
        }
    }
}



struct ChunkListNode<B: Backend, A: Allocator<B>> {
    id: usize,
    chunks_per_block: usize,
    chunk_size: usize,
    blocks: Vec<(Block<B, A::Tag>, usize)>,
    free: VecDeque<(usize, usize)>,
}


impl<B, A> ChunkListNode<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    fn new(chunk_size: usize, chunks_per_block: usize, id: usize) -> Self {
        ChunkListNode {
            id,
            chunk_size,
            chunks_per_block,
            blocks: Vec::new(),
            free: VecDeque::new(),
        }
    }

    fn count(&self) -> usize {
        self.block.len() * self.chunks_per_block
    }

    fn grow(&mut self, owner: &mut A, device: &B::Device, info: A::Info) -> Result<(), A::Error> {
        assert!(owner.align() >= self.chunk_size);
        let reqs = Requirements {
            type_mask: 1 << self.id,
            size: self.chunk_size * self.chunks_per_block,
            alignment: self.chunk_size,
        };
        let block = owner.alloc(device, info, reqs)?;
        let shift = calc_alignment_shift(reqs.alignment, block.offset);

        for i in 0..((block.size - shift) / self.chunk_size) {
            self.free.push((self.blocks.len(), i));
        }

        self.blocks.push((block, shift));

        Ok(())
    }

    fn alloc_no_grow(&mut self, owner: &mut A, device: &B::Device) -> Option<Block<B, usize>> {
        self.free.pop_front().map(|(block_index, chunk_index)|) {
            return Ok(Block {
                memory: self.blocks[block_index].memory,
                tag: block_index,
                offset: self.blocks[block_index].offset + chunk_index * self.chunk_size,
                size: self.chunk_size,
            });
        })
    }
}

impl<B, A> Relevant for ChunkListNode<B, A> where B: Backend, A: Allocator<B>, {}


impl<B, A> SubAllocator<B> for ChunkListNode<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    type Owner = A;
    type Info = A::Info;
    type Tag = usize;
    type Error = A::Error;


    fn alloc(&mut self, owner: &mut A, device: &B::Device, info: A::Info, reqs: Requirements) -> Result<Block<B, usize>, A::Error> {
        assert!((1 << self.id) & reqs.type_mask);
        assert!(self.chunk_size >= reqs.size);
        assert!(self.chunk_size >= reqs.alignment);
        if let Some(block) = self.alloc_no_grow(owner, device) {
            block
        } else {
            self.grow(owner, device, info)?;
            self.alloc_no_grow(owner, device).unwrap()
        }
    }

    fn free(&mut self, _: &B::Device, block: Block<B, usize>) {
        assert_eq!(block.offset % self.chunk_size, 0);
        let block_index = block.tag;
        let offset = (block.offset - self.blocks[block_index].offset);
        assert_eq!(offset % self.chunk_size, 0);
        let chunk_index = offset / self.chunk_size;
        self.free.push_front((block_index, chunk_index));
        forget(block);
    }

    fn is_unused(&self) -> bool {
        self.count() == self.free.len()
    }

    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert_eq!(self.is_unused());
        unsafe {
            for block in self.blocks.drain(..) {
                owner.free(device, block);
            }
            drop(read(&mut self.blocks));
            drop(read(&mut self.free));
            forget(self);
        }
    }
}




struct ChunkListAllocator<B: Backend, A: Allocator<B>> {
    chunk_size_bit: usize,
    min_size_bit: usize,
    nodes: Vec<ChunkListNode<B, A>>,
}

impl<B, A> ChunkListAllocator<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    fn new(chunk_size_bit: usize, min_size_bit: usize, id: usize) -> Self {
        ChunkListAllocator {
            id,
            chunk_size_bit,
            min_size_bit,
            nodes: Vec::with_capacity(32),
        }
    }

    fn pick_node(&self, size: usize) -> usize {
        let bits = ::std::mem::size_of::<usize>() * 8;
        assert!(size != 0);
        bits - ((size - 1) >> self.min_size_bit).leading_zeros()
    }

    fn chunk_size(&self, index: usize) -> usize {
        1 << (self.min_size_bit + index);
    }

    fn chunks_per_block(&self) -> usize {
        1 << self.chunk_size_bit
    }

    fn chunk_index(&self, index: usize, offset: usize) -> usize {
        assert!(offset % self.chunk_size(index), 0);
        offset / self.chunk_size(index)
    }

    fn grow(&mut self, owner: &mut A, size: usize) {
        let len = self.nodes.len();
        self.nodes.len().extend((len .. size).map(|index| {
            ChunkListNode::new(self.chunk_size(index), self.chunks_per_block(), self.id)
        }));
    }
}

impl<B, A> Relevant for ChunkListAllocator<B, A> where B: Backend, A: Allocator<B>, {}

impl<B, A> SubAllocator<B> for ChunkListAllocator<B, A>
where
    B: Backend,
    A: Allocator<B>,
{
    type Owner = A;
    type Info = A::Info;
    type Tag = usize;
    type Error = A::Error;

    fn alloc(&mut self, owner: &mut A, device: &B::Device, info: A::Info, reqs: Requirements) -> Result<Block<B, usize>, A::Error> {
        let index = self.pick_node(size);
        self.grow(owner, index + 1);
        self.nodes[index].alloc(owner, device, info, reqs)
    }
    fn free(&mut self, owner: &mut A, device: &B::Device, block: Block<B, usize>) {
        let index = self.pick_node(size);
        self.nodes[index].free(owner, device, block);
    }
    fn is_unused(&self) -> bool {
        self.nodes.iter().all(ChunkListNode::is_unused)
    }

    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert_eq!(self.is_unused());
        unsafe {
            for node in self.nodes.drain(..) {
                node.dispose(owner, device);
            }
            read(&mut self.nodes);
            forget(self);
        }
    }
}


pub struct MemoryAllocator {
    memory_type: MemoryType,
    allocations: usize,
}

impl MemoryAllocator {
    fn new(memory_type: MemoryType) -> Self {
        MemoryAllocator {
            memory_type,
            allocations,
        }
    }

    pub fn memory_type(&self) -> &MemoryType {
        &self.memory_type
    }
}

impl Relevant for MemoryAllocator {}

impl<B> Allocator<B> for MemoryAllocator
where
    B: Backend,
{
    type Tag = *mut Self;
    type Info = ();
    type Error = OutOfMemory;

    fn align(&self, _: (), size: usize) -> usize {
        size
    }
    fn alloc(&mut self, device: &B::Device, _: (), reqs: Requirements) -> Result<Block<B, Self::Tag>, A::Error> {
        assert!((1 << self.memory_type.id) & reqs.type_mask);
        let memory = device.allocate_memory(self.memory_type, reqs.size)?;
        let memory = Box::into_raw(Box::new(memory)); // Unoptimal
        self.allocations += 1;
        Ok(Block {
            memory,
            offset: 0,
            size: size,
            tag: self as *mut Self,
        })
    }
    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>) {
        assert_eq!(block.offset, 0);
        assert_eq!(block.tag == self as *mut Self);
        device.free_memory(*unsafe { Box::from_raw(block.memory) })
        forget(block);
        self.allocations -= 1;
    }
    fn is_unused(&self) -> bool {
        self.allocations == 0
    }

    fn dispose(self, _: &B::Device) {
        assert!(self.is_unused());
        forget(self);
    }
}


pub enum AllocationType {
    Arena,
    Chunk,
}

pub struct CombinedAllocator<B>
where
    B: Backend,
{
    memory: MemoryAllocator,
    arenas: ArenaAllocator<B, MemoryAllocator>,
    chunks: ChunkListAllocator<B, MemoryAllocator>,
}

impl<B> CombinedAllocator<B>
where
    B: Backend,
{
    pub fn new(memory_type: MemoryType, arena_size: usize, chunk_size_bit: usize, min_size_bit: usize) -> Self {
        CombinedAllocator {
            memory: MemoryAllocator::new(memory_type),
            arenas: ArenaAllocator::new(arena_size),
            chunks: ChunkListAllocator::new(chunk_size_bit, min_size_bit),
        }
    }

    pub fn memory_type(&self) -> &MemoryType {
        self.memory.memory_type()
    }
}


impl<B> Relevant for CombinedAllocator<B> where B: Backend {}

impl<B> Allocator<B> for CombinedAllocator<B>
where
    B: Backend,
{
    type Info = AllocationType;
    type Tag = (AllocationType, usize);
    type Error = OutOfMemory;

    fn alloc(&mut self, device: &B::Device, reqs: Requirements) -> Result<Block<B, AllocationType>, Self::Error> {
        match info {
            AllocationType::Arena => {
                self.arenas.alloc(&mut self.memory, device, reqs).map(|block| block.convert(|value| (AllocationType::Arena, value)))
            }
            AllocationType::Chunk => {
                self.chunks.alloc(&mut self.memory, device, reqs).map(|block| block.convert(|value| (AllocationType::Chunk, value))),
            }
        }
        
    }

    fn free(&mut, device: &B::Device, block: Block<B, AllocationType>) {
        let type = block.tag.0;
        let block = block.convert(|(value, _)| value);
        match block.tag.0 {
            AllocationType::Arena => {
                self.arenas.free(&mut self.memory, device, block)
            }
            AllocationType::Chunk => {
                self.chunks.free(&mut self.memory, device, block)
            }
        }
    }

    fn is_unused(&self) -> bool {
        self.arenas.is_unused() && self.chunks.is_unused()
    }
    
    fn dispose(mut self, device: &B::Device) {
        read(&mut self.arenas).dispose(&mut self.memory, device);
        read(&mut self.chunks).dispose(&mut self.memory, device);
        read(&mut self.memory).dispose();
        forget(self);
    }
}


pub struct SmartAllocator<B: Backend> {
    allocators: Vec<CombinedAllocator<B>>,
}

impl<B> Relevant for SmartAllocator<B> where B: Backend {}

impl<B> Allocator<B> for SmartAllocator<B>
where
    B: Backend,
{
    type Info = (AllocationType, Properties);
    type Tag = (usize, AllocatorType, usize);
    type Error = OutOfMemory;

    fn alloc(&mut self, device: &B::Device, (type, prop): (AllocationType, Properties), reqs: Requirements) -> Result<Block<B, Self::Tag>, OutOfMemory> {
        let (index, ref allocator) = *self.allocators.iter_mut().enumerate().find(|(index, allocator)| {
            let memory_type = allocator.memory_type();
            ((1 << memory_type.id) reqs.type_mask) && memory_type.properties.contains(prop)
        }).unwrap();
        let block = allocators.alloc(device, type, reqs);
        block.convert(|(type, value)| (index, type, value))
    }

    fn free(&mut self, device: &B::Device, block: Block<B, Self::Tag>) {
        let index = block.tag.0;
        let block = block.convert(|(_, type, value)| (type, value));
        self.allocators[index].free(device, block);
    }

    fn is_unused(&self) {
        self.allocators.all(Allocator::is_unused)
    }

    fn dispose(mut self, device: &B::Device) {
        for allocator in self.allocators.drain(..) {
            allocator.dispose(device);
        }
        read(&mut self.allocators);
        forget(self);
    }
}

fn calc_alignment_shift(alignment: usize, offset: usize) -> usize {
    if offset == 0 {
        0
    } else {
        alignment - (offset - 1) % alignment - 1
    }
}


fn shift_for_alignment(alignment: usize, offset: usize) -> usize {
    offset + calc_alignment_shift(alignment, offset)
}