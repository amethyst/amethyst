use std::collections::VecDeque;
use std::mem::replace;

use gfx_hal::Backend;
use gfx_hal::memory::Requirements;


use memory::{calc_alignment_shift, Block, MemoryAllocator, MemorySubAllocator, Result};


#[derive(Debug)]
struct ArenaNode<B: Backend, A: MemoryAllocator<B>> {
    block: Block<B, A::Tag>,
    used: u64,
    freed: u64,
}

impl<B, A> ArenaNode<B, A>
where
    B: Backend,
    A: MemoryAllocator<B>,
{
    fn new(block: Block<B, A::Tag>) -> Self {
        ArenaNode {
            block,
            used: 0,
            freed: 0,
        }
    }

    fn alloc(&mut self, reqs: Requirements) -> Option<Block<B, ()>> {
        let offset = self.block.range().start + self.used;
        let total_size = reqs.size + calc_alignment_shift(reqs.alignment, offset);

        if self.block.size() - self.used < total_size {
            None
        } else {
            self.used += total_size;
            Some(Block::new(self.block.memory(), offset..total_size + offset))
        }
    }

    fn free(&mut self, block: Block<B, ()>) {
        assert!(self.block.contains(&block));
        self.freed += block.size();
        unsafe { block.dispose() }
    }

    fn is_unused(&self) -> bool {
        self.freed == self.used
    }

    fn dispose(self, owner: &mut A, device: &B::Device) {
        assert!(self.is_unused());
        let ArenaNode { block, .. } = self;
        owner.free(device, block);
    }
}


/// Linear allocator for transient memory
#[derive(Debug)]
pub struct ArenaAllocator<B: Backend, A: MemoryAllocator<B>> {
    id: usize,
    arena_size: u64,
    hot: Option<Box<ArenaNode<B, A>>>,
    freed: usize,
    nodes: VecDeque<Box<ArenaNode<B, A>>>,
}

impl<B, A> ArenaAllocator<B, A>
where
    B: Backend,
    A: MemoryAllocator<B>,
{
    /// Construct allocator.
    pub fn new(arena_size: u64, id: usize) -> Self {
        ArenaAllocator {
            id,
            arena_size,
            hot: None,
            freed: 0,
            nodes: VecDeque::new(),
        }
    }

    fn cleanup(&mut self, owner: &mut A, device: &B::Device) {
        while self.nodes
            .front()
            .map(|node| node.is_unused())
            .unwrap_or(false)
        {
            self.nodes
                .pop_front()
                .map(|node| node.dispose(owner, device));
            self.freed += 1;
        }
    }

    fn allocate_node(
        &mut self,
        owner: &mut A,
        device: &B::Device,
        info: A::Info,
        reqs: Requirements,
    ) -> Result<ArenaNode<B, A>> {
        let arena_size = ((reqs.size - 1) / self.arena_size + 1) * self.arena_size;
        let arena_requirements = Requirements {
            type_mask: 1 << self.id,
            size: arena_size,
            alignment: reqs.alignment,
        };
        let arena_block = owner.alloc(device, info, arena_requirements)?;
        Ok(ArenaNode::new(arena_block))
    }
}

impl<B, A> MemorySubAllocator<B> for ArenaAllocator<B, A>
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
    ) -> Result<Block<B, Self::Tag>> {
        assert_eq!((1 << self.id) & reqs.type_mask, 1 << self.id);
        let count = self.nodes.len();
        if let Some(ref mut hot) = self.hot.as_mut() {
            match hot.alloc(reqs) {
                Some(block) => return Ok(block.set_tag(count)),
                None => {}
            }
        };

        let mut node = self.allocate_node(owner, device, info, reqs)?;
        let block = node.alloc(reqs).unwrap();
        if let Some(hot) = replace(&mut self.hot, Some(Box::new(node))) {
            if hot.is_unused() {
                hot.dispose(owner, device);
            } else {
                self.nodes.push_back(hot)
            }
        };
        let count = self.nodes.len();
        Ok(block.set_tag(count))
    }

    fn free(&mut self, owner: &mut A, device: &B::Device, block: Block<B, Self::Tag>) {
        let (block, tag) = block.replace_tag(());
        let index = tag - self.freed;

        match self.nodes.len() {
            len if len == index => {
                self.hot.as_mut().unwrap().free(block);
            }
            len if len > index => {
                self.nodes[index].free(block);
                self.cleanup(owner, device);
            }
            _ => unreachable!(),
        }
    }

    fn is_unused(&self) -> bool {
        self.nodes.is_empty()
            && self.hot
                .as_ref()
                .map(|node| node.is_unused())
                .unwrap_or(true)
    }

    fn dispose(mut self, owner: &mut A, device: &B::Device) {
        assert!(self.is_unused());
        if let Some(hot) = self.hot.take() {
            hot.dispose(owner, device);
        }
    }
}
