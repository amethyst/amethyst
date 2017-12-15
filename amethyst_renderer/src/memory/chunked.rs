use std::collections::VecDeque;

use gfx_hal::{Backend};
use gfx_hal::memory::Requirements;

use memory::{shift_for_alignment, Block, MemoryAllocator, MemorySubAllocator, Result};

#[derive(Debug)]
struct ChunkListNode<B: Backend, A: MemoryAllocator<B>> {
    id: usize,
    chunks_per_block: usize,
    chunk_size: u64,
    blocks: Vec<(Block<B, A::Tag>, u64)>,
    free: VecDeque<(usize, u64)>,
}


impl<B, A> ChunkListNode<B, A>
where
    B: Backend,
    A: MemoryAllocator<B>,
{
    fn new(chunk_size: u64, chunks_per_block: usize, id: usize) -> Self {
        ChunkListNode {
            id,
            chunk_size,
            chunks_per_block,
            blocks: Vec::new(),
            free: VecDeque::new(),
        }
    }

    fn count(&self) -> usize {
        self.blocks.len() * self.chunks_per_block
    }

    fn grow(&mut self, owner: &mut A, device: &B::Device, info: A::Info) -> Result<()> {
        let reqs = Requirements {
            type_mask: 1 << self.id,
            size: self.chunk_size * self.chunks_per_block as u64,
            alignment: self.chunk_size,
        };
        let block = owner.alloc(device, info, reqs)?;
        let offset = shift_for_alignment(reqs.alignment, block.offset);

        assert!(self.chunks_per_block as u64 <= (block.size - offset) / self.chunk_size);

        for i in 0..self.chunks_per_block as u64 {
            self.free.push_back((self.blocks.len(), i));
        }
        self.blocks.push((block, offset));

        Ok(())
    }

    fn alloc_no_grow(&mut self) -> Option<Block<B, usize>> {
        self.free.pop_front().map(|(block_index, chunk_index)| {
            let offset = self.blocks[block_index].1 + chunk_index * self.chunk_size;
            let block = Block::new(
                self.blocks[block_index].0.memory,
                offset..self.chunk_size + offset,
            );
            block.set_tag(block_index)
        })
    }
}


impl<B, A> MemorySubAllocator<B> for ChunkListNode<B, A>
where
    B: Backend,
    A: MemoryAllocator<B>,
{
    type Owner = A;
    type Info = A::Info;
    type Tag = usize;

    fn alloc(
        &mut self,
        owner: &mut A,
        device: &B::Device,
        info: A::Info,
        reqs: Requirements,
    ) -> Result<Block<B, usize>> {
        assert_eq!((1 << self.id) & reqs.type_mask, 1 << self.id);
        assert!(self.chunk_size >= reqs.size);
        assert!(self.chunk_size >= reqs.alignment);
        if let Some(block) = self.alloc_no_grow() {
            Ok(block)
        } else {
            self.grow(owner, device, info)?;
            Ok(self.alloc_no_grow().unwrap())
        }
    }

    fn free(&mut self, _owner: &mut A, _device: &B::Device, block: Block<B, usize>) {
        assert_eq!(block.offset % self.chunk_size, 0);
        assert_eq!(block.size, self.chunk_size);
        let offset = block.offset;
        let block_index = unsafe { block.dispose() };
        let offset = offset - self.blocks[block_index].1;
        let chunk_index = offset / self.chunk_size;
        self.free.push_front((block_index, chunk_index));
    }

    fn is_unused(&self) -> bool {
        self.count() == self.free.len()
    }

    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert!(self.is_unused());
        for (block, _) in self.blocks.drain(..) {
            owner.free(device, block);
        }
    }
}


#[derive(Debug)]
pub struct ChunkListAllocator<B: Backend, A: MemoryAllocator<B>> {
    id: usize,
    chunk_size_bit: u8,
    min_size_bit: u8,
    nodes: Vec<ChunkListNode<B, A>>,
}

impl<B, A> ChunkListAllocator<B, A>
where
    B: Backend,
    A: MemoryAllocator<B>,
{
    /// # Panics
    ///
    /// Panics if `chunk_size` or `min_chunk_size` is not power of 2.
    ///
    pub fn new(chunk_size: u64, min_chunk_size: u64, id: usize) -> Self {
        let bits = (::std::mem::size_of::<usize>() * 8) as u32;
        let chunk_size_bit = (bits - chunk_size.leading_zeros() - 1) as u8;
        assert_eq!(1u64 << chunk_size_bit, chunk_size);
        let min_size_bit = (bits - min_chunk_size.leading_zeros() - 1) as u8;
        assert_eq!(1u64 << min_size_bit, min_chunk_size);
        ChunkListAllocator {
            id,
            chunk_size_bit,
            min_size_bit,
            nodes: Vec::with_capacity(16),
        }
    }

    fn pick_node(&self, size: u64) -> u8 {
        let bits = ::std::mem::size_of::<usize>() * 8;
        assert!(size != 0);
        (bits - ((size - 1) >> self.min_size_bit).leading_zeros() as usize) as u8
    }

    fn grow(&mut self, size: u8) {
        let min_size_bit = self.min_size_bit;
        let chunk_size = |index: u8| 1u64 << (min_size_bit + index as u8);
        let chunks_per_block = 1 << self.chunk_size_bit;
        let id = self.id;
        let len = self.nodes.len() as u8;
        self.nodes.extend((len..size).map(|index| {
            ChunkListNode::new(chunk_size(index), chunks_per_block, id)
        }));
    }
}


impl<B, A> MemorySubAllocator<B> for ChunkListAllocator<B, A>
where
    B: Backend,
    A: MemoryAllocator<B>,
{
    type Owner = A;
    type Info = A::Info;
    type Tag = usize;

    fn alloc(
        &mut self,
        owner: &mut A,
        device: &B::Device,
        info: A::Info,
        reqs: Requirements,
    ) -> Result<Block<B, usize>> {
        let index = self.pick_node(reqs.size);
        self.grow(index + 1);
        self.nodes[index as usize].alloc(owner, device, info, reqs)
    }
    fn free(&mut self, owner: &mut A, device: &B::Device, block: Block<B, usize>) {
        let index = self.pick_node(block.size);
        self.nodes[index as usize].free(owner, device, block);
    }
    fn is_unused(&self) -> bool {
        self.nodes.iter().all(ChunkListNode::is_unused)
    }

    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert!(self.is_unused());
        for node in self.nodes.drain(..) {
            node.dispose(owner, device);
        }
    }
}
