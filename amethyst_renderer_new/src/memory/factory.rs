

use std::cmp::{Ordering, PartialOrd, max, min};
use std::collections::VecDeque;
use std::ops::{Add, Deref, DerefMut};


use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::memory::{Properties, Requirements};
use gfx_hal::buffer::{Usage as BufferUsage, complete_requirements};
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::device::OutOfMemory;
use gfx_hal::mapping::Error as MappingError;

use specs::{Fetch, FetchMut};


use epoch::{CurrentEpoch, DeletionQueue, Eh, Epoch, ValidThrough};
use memory::{Allocator, Block, Result, SmartAllocator,
             shift_for_alignment};
use memory::combined::Type as AllocationType;
use relevant::Relevant;

pub type Buffer<B: Backend> = Eh<Item<B, B::Buffer>>;
pub type Image<B: Backend> = Eh<Item<B, B::Image>>;

#[derive(Debug)]
pub struct Item<B: Backend, T> {
    inner: T,
    block: Block<B, <SmartAllocator<B> as Allocator<B>>::Tag>,
    properties: Properties,
    requirements: Requirements,
}

impl<B, T> Deref for Item<B, T>
where
    B: Backend,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<B, T> DerefMut for Item<B, T>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

pub struct Factory<B: Backend> {
    allocator: SmartAllocator<B>,
    buffer_deletion_queue: DeletionQueue<Eh<Item<B, B::Buffer>>>,
    image_deletion_queue: DeletionQueue<Eh<Item<B, B::Image>>>,
}

impl<B> Factory<B>
where
    B: Backend,
{
    pub fn new(
        memory_types: Vec<MemoryType>,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        Factory {
            allocator: SmartAllocator::new(memory_types, arena_size, chunk_size, min_chunk_size),
            buffer_deletion_queue: DeletionQueue::new(),
            image_deletion_queue: DeletionQueue::new(),
        }
    }

    pub fn create_buffer(
        &mut self,
        device: &B::Device,
        size: u64,
        stride: u64,
        usage: BufferUsage,
        properties: Properties,
        transient: bool,
    ) -> Result<Buffer<B>> {
        let ubuf = device.create_buffer(size, stride, usage)?;
        let requirements = complete_requirements::<B>(device, &ubuf, usage);
        let ty = if transient {
            AllocationType::Arena
        } else {
            AllocationType::Chunk
        };
        let block = self.allocator.alloc(device, (ty, properties), requirements)?;
        let buf = device
            .bind_buffer_memory(
                block.memory(),
                shift_for_alignment(requirements.alignment, block.range().start),
                ubuf,
            )
            .unwrap();
        Ok(Eh::new(Item {
            inner: buf,
            block,
            properties,
            requirements,
        }))
    }

    pub fn create_image(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
        properties: Properties,
    ) -> Result<Image<B>> {
        let uimg = device.create_image(kind, level, format, usage)?;
        let requirements = device.get_image_requirements(&uimg);
        let block = self.allocator.alloc(
            device,
            (AllocationType::Chunk, properties),
            requirements,
        )?;
        let img = device
            .bind_image_memory(
                block.memory(),
                shift_for_alignment(requirements.alignment, block.range().start),
                uimg,
            )
            .unwrap();
        Ok(Eh::new(Item {
            inner: img,
            block,
            properties,
            requirements,
        }))
    }

    pub fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.buffer_deletion_queue.add(buffer);
    }

    pub fn destroy_image(&mut self, image: Image<B>) {
        self.image_deletion_queue.add(image);
    }

    pub fn cleanup(&mut self, device: &B::Device, current: &CurrentEpoch) {
        let ref mut allocator = self.allocator;
        self.image_deletion_queue.clean(current, |image| {
            device.destroy_image(image.inner);
            allocator.free(device, image.block);
        });
        self.buffer_deletion_queue.clean(current, |buffer| {
            device.destroy_buffer(buffer.inner);
            allocator.free(device, buffer.block);
        });
    }
}
