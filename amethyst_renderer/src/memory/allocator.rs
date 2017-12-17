use gfx_hal::{Backend, Device, MemoryType};
use gfx_hal::buffer::{complete_requirements, Usage as BufferUsage};
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::memory::{Properties, Requirements};

use epoch::{CurrentEpoch, DeletionQueue, Ec, Eh};
use memory::{shift_for_alignment, Block, ErrorKind, MemoryAllocator, Result, SmartAllocator};
use memory::combined::Type as AllocationType;

pub type Buffer<B: Backend> = Eh<Item<B, B::Buffer>>;
pub type Image<B: Backend> = Eh<Item<B, B::Image>>;

pub type WeakBuffer<B: Backend> = Ec<Item<B, B::Buffer>>;
pub type WeakImage<B: Backend> = Ec<Item<B, B::Image>>;

type BlockTag<B: Backend> = <SmartAllocator<B> as MemoryAllocator<B>>::Tag;

#[derive(Debug)]
pub struct Item<B: Backend, T> {
    inner: T,
    block: Block<B, BlockTag<B>>,
    properties: Properties,
    requirements: Requirements,
}

impl<B, T> Item<B, T>
where
    B: Backend,
{
    pub fn get_alignment(&self) -> u64 {
        self.requirements.alignment
    }

    pub fn get_size(&self) -> u64 {
        self.requirements.size
    }

    pub fn visible(&self) -> bool {
        self.properties.contains(Properties::CPU_VISIBLE)
    }

    pub fn raw(&self) -> &T {
        &self.inner
    }
}

unsafe impl<B, T> Send for Item<B, T>
where
    B: Backend,
    T: Send,
{
}
unsafe impl<B, T> Sync for Item<B, T>
where
    B: Backend,
    T: Sync,
{
}

pub struct Allocator<B: Backend> {
    allocator: SmartAllocator<B>,
    buffer_deletion_queue: DeletionQueue<Eh<Item<B, B::Buffer>>>,
    image_deletion_queue: DeletionQueue<Eh<Item<B, B::Image>>>,
}

impl<B> Allocator<B>
where
    B: Backend,
{
    pub fn new(
        memory_types: Vec<MemoryType>,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        Allocator {
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
        mut properties: Properties,
        transient: bool,
    ) -> Result<Buffer<B>> {
        // Remove this when metal will support buffer copy operation.
        #[cfg(feature = "gfx-backend-metal")]
        let properties = {
            use std::any::Any;
            if Any::is::<<::metal::Backend as Backend>::Device>(device) {
                let mut properties = properties;
                properties.insert(Properties::CPU_VISIBLE);
                properties.remove(Properties::DEVICE_LOCAL);
                properties
            } else {
                properties
            }
        };

        let ubuf = device.create_buffer(size, stride, usage)?;
        let requirements = complete_requirements::<B>(device, &ubuf, usage);
        let ty = if transient {
            AllocationType::Arena
        } else {
            AllocationType::Chunk
        };
        let block = self.allocator
            .alloc(device, (ty, properties), requirements)
            .map_err(|err| {
                if let &ErrorKind::NoCompatibleMemoryType = err.kind() {
                    ErrorKind::BufferUsageAndProperties(usage, properties).into()
                } else {
                    err
                }
            })?;
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
        let ty = AllocationType::Chunk;
        let block = self.allocator
            .alloc(device, (ty, properties), requirements)
            .map_err(|err| {
                if let &ErrorKind::NoCompatibleMemoryType = err.kind() {
                    ErrorKind::ImageUsageAndProperties(usage, properties).into()
                } else {
                    err
                }
            })?;
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
