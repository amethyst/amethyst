

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


use epoch::{DeletionQueue, Epoch, CurrentEpoch, Eh, ValidThrough};
use memory::{Allocator, Block, MemoryError, MemoryErrorKind, SmartAllocator, MemoryResult, shift_for_alignment};
use memory::combined::Type as AllocationType;
use relevant::Relevant;

pub struct Piece<B: Backend, T> {
    inner: T,
    block: Block<B, <SmartAllocator<B> as Allocator<B>>::Tag>,
    properties: Properties,
    requirements: Requirements,
}

impl<B, T> Deref for Piece<B, T>
where
    B: Backend,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<B, T> DerefMut for Piece<B, T>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

fn create_buffer<B: Backend>(
    allocator: &mut SmartAllocator<B>,
    device: &B::Device,
    size: u64,
    stride: u64,
    usage: BufferUsage,
    properties: Properties,
    transient: bool,
) -> MemoryResult<Piece<B, B::Buffer>> {
    let ubuf = device.create_buffer(size, stride, usage)?;
    let requirements = complete_requirements::<B>(device, &ubuf, usage);
    let ty = if transient {
        AllocationType::Arena
    } else {
        AllocationType::Chunk
    };
    let block = allocator.alloc(device, (ty, properties), requirements)?;
    let buf = device
        .bind_buffer_memory(
            block.memory(),
            shift_for_alignment(requirements.alignment, block.range().start),
            ubuf,
        )
        .unwrap();
    Ok(Piece {
        inner: buf,
        block,
        properties,
        requirements,
    })
}
fn create_image<B: Backend>(
    allocator: &mut SmartAllocator<B>,
    device: &B::Device,
    kind: Kind,
    level: Level,
    format: Format,
    usage: ImageUsage,
    properties: Properties,
) -> MemoryResult<Piece<B, B::Image>> {
    let uimg = device.create_image(kind, level, format, usage)?;
    let requirements = device.get_image_requirements(&uimg);
    let block = allocator.alloc(
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
    Ok(Piece {
        inner: img,
        block,
        properties,
        requirements,
    })
}
fn destroy_buffer<B: Backend>(
    allocator: &mut SmartAllocator<B>,
    device: &B::Device,
    buffer: Piece<B, B::Buffer>,
) {
    device.destroy_buffer(buffer.inner);
    allocator.free(device, buffer.block);
}
fn destroy_image<B: Backend>(
    allocator: &mut SmartAllocator<B>,
    device: &B::Device,
    image: Piece<B, B::Image>,
) {
    device.destroy_image(image.inner);
    allocator.free(device, image.block);
}

pub struct Item<B: Backend, T> {
    item: Piece<B, T>,
    valid_through: Epoch,
}

impl<B, T> Item<B, T>
where
    B: Backend,
{
    /// Make all new `Ec` borrowed this to be valid
    /// until specifyed `Epoch` expired
    pub fn make_valid_through(this: &mut Self, epoch: Epoch) {
        this.valid_through = max(this.valid_through, epoch);
    }

    /// Convert `Item` into `Eh` so that user can get shared
    /// reference to it as `Ec`
    pub fn into_shared(this: Self) -> Eh<Piece<B, T>> {
        let mut eh = Eh::new(this.item);
        Eh::make_valid_through(&mut eh, this.valid_through);
        eh
    }

    pub fn from_shared(shared: Eh<Piece<B, T>>, current: &CurrentEpoch) -> Result<Self, Eh<Piece<B, T>>> {
        shared.dispose(current).map(|item| {
            Item {
                item,
                valid_through: Epoch::new(),
            }
        })
    }
}

impl<B, T> ValidThrough for Item<B, T>
where
    B: Backend,
{
    type Data = Piece<B, T>;

    fn valid_through(&self) -> Epoch {
        self.valid_through
    }

    fn dispose(self, current: &CurrentEpoch) -> Result<Piece<B, T>, Self> {
        if self.valid_through < current.now() {
            Ok(self.item)
        } else {
            Err(self)
        }
    }
}

pub type Buffer<B: Backend> = Item<B, B::Buffer>;
pub type Image<B: Backend> = Item<B, B::Image>;


impl<B, T> Deref for Item<B, T>
where
    B: Backend,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.item.inner
    }
}

impl<B, T> DerefMut for Item<B, T>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.item.inner
    }
}

pub struct Factory<B: Backend> {
    allocator: SmartAllocator<B>,
    buffer_deletion_queue: DeletionQueue<Item<B, B::Buffer>>,
    image_deletion_queue: DeletionQueue<Item<B, B::Image>>,
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
    ) -> MemoryResult<Buffer<B>> {
        let buffer = create_buffer(
            &mut self.allocator,
            device,
            size,
            stride,
            usage,
            properties,
            transient,
        )?;
        Ok(Item {
            item: buffer,
            valid_through: Epoch::new(),
        })
    }

    pub fn create_image(
        &mut self,
        device: &B::Device,
        kind: Kind,
        level: Level,
        format: Format,
        usage: ImageUsage,
        properties: Properties,
    ) -> MemoryResult<Image<B>> {
        let image = create_image(
            &mut self.allocator,
            device,
            kind,
            level,
            format,
            usage,
            properties,
        )?;
        Ok(Item {
            item: image,
            valid_through: Epoch::new(),
        })
    }

    pub fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.buffer_deletion_queue.add(buffer);
    }

    pub fn destroy_image(&mut self, image: Image<B>) {
        self.image_deletion_queue.add(image);
    }

    pub fn cleanup(&mut self, device: &B::Device, current: &CurrentEpoch) {
        let ref mut allocator = self.allocator;
        self.image_deletion_queue.clean(
            current,
            |image| { destroy_image(allocator, device, image); },
        );
        self.buffer_deletion_queue.clean(current, |buffer| {
            destroy_buffer(allocator, device, buffer);
        });
    }
}
