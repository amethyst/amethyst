
use std::ops::{Deref, DerefMut};

use gfx_hal::{Backend, Device, MemoryProperties};
use gfx_hal::buffer::{Usage as BufferUsage};
use gfx_hal::format::Format;
use gfx_hal::image::{Kind, Level, Usage as ImageUsage};
use gfx_hal::mapping::Error as MappingError;
use gfx_hal::memory::{Properties, Requirements, Pod};

use epoch::{CurrentEpoch, DeletionQueue, Ec, Eh};
use memory::{shift_for_alignment, Block, Error, MemoryAllocator};
use memory::combined::Type as AllocationType;
use memory::smart::SmartAllocator;

pub type Buffer<B: Backend> = Eh<Item<B, B::Buffer>>;
pub type Image<B: Backend> = Eh<Item<B, B::Image>>;

pub type WeakBuffer<B: Backend> = Ec<Item<B, B::Buffer>>;
pub type WeakImage<B: Backend> = Ec<Item<B, B::Image>>;

type BlockTag<B: Backend> = <SmartAllocator<B> as MemoryAllocator<B>>::Tag;

/// Item bound to the block of memory.
/// It can be `Image`, `Buffer` or something exotic.
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
    /// Get alignment of the underlying memory block
    pub fn get_alignment(&self) -> u64 {
        self.requirements.alignment
    }

    /// Get size of the underlying memory block
    pub fn get_size(&self) -> u64 {
        self.block.size()
    }

    /// Check if the underlying memory block is visible by cpu
    pub fn visible(&self) -> bool {
        self.properties.contains(Properties::CPU_VISIBLE)
    }

    /// Check if the underlying memory block is coherent
    /// between host and device.
    pub fn coherent(&self) -> bool {
        self.properties.contains(Properties::COHERENT)
    }

    /// Get row inner object.
    pub fn raw(&self) -> &T {
        &self.inner
    }

    /// Acquire reader for range of the underlying memory block.
    /// Reader can be dereferenced to slice.
    /// It will automatically unmap memory when dropped.
    pub fn read<'a>(&'a mut self, offset: u64, size: usize, device: &'a B::Device) -> Result<Reader<'a, B>, MappingError> {
        use std::mem::size_of;

        let bytes = size as u64;

        if !self.visible() {
            return Err(MappingError::InvalidAccess);
        }
        let start = self.block.range().start + offset;
        let end = start + bytes;
        if self.block.range().end < end {
            return Err(MappingError::OutOfBounds);
        }
        let range = start..end;

        let memory = self.block.memory();
        let ptr = device.map_memory(memory, range.clone())?;
        if !self.coherent() {
            device.invalidate_mapped_memory_ranges(&[(memory, range.clone())]);
        }
        Ok(Reader {
            ptr,
            size,
            memory,
            device,
        })
    }

    /// Acquire writer for range of the underlying memory block.
    /// Writer can be dereferenced to mutable slice.
    /// It will automatically flush and unmap memory when dropped.
    /// Flushing do not occur if writer wasn't dereferenced even once.
    pub fn write<'a>(&'a self, fresh: bool, offset: u64, size: usize, device: &'a B::Device) -> Result<Writer<'a, B>, MappingError> {
        use std::mem::size_of;

        let bytes = size as u64;

        if !self.visible() {
            return Err(MappingError::InvalidAccess);
        }
        let start = self.block.range().start + offset;
        let end = start + bytes;
        if self.block.range().end < end {
            return Err(MappingError::OutOfBounds);
        }
        let range = start..end;

        let memory = self.block.memory();
        let ptr = device.map_memory(memory, range.clone())?;
        if !fresh && !self.coherent() {
            device.invalidate_mapped_memory_ranges(&[(memory, range.clone())]);
        }
        Ok(Writer {
            coherent: self.coherent(),
            flushed: false,
            ptr,
            start,
            size,
            memory,
            device,
        })
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

pub struct Reader<'a, B: Backend> {
    ptr: *const u8,
    size: usize,
    memory: &'a B::Memory,
    device: &'a B::Device,
}

impl<'a, B> Deref for Reader<'a, B>
where
    B: Backend,
{
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        use std::slice::from_raw_parts;
        unsafe {
            from_raw_parts(self.ptr, self.size)
        }
    }
}

impl<'a, B> Drop for Reader<'a, B>
where
    B: Backend,
{
    fn drop(&mut self) {
        self.device.unmap_memory(self.memory);
    }
}

pub struct Writer<'a, B: Backend> {
    coherent: bool,
    flushed: bool,
    ptr: *mut u8,
    start: u64,
    size: usize,
    memory: &'a B::Memory,
    device: &'a B::Device,
}

impl<'a, B> Writer<'a, B>
where
    B: Backend,
{
    pub fn flush(&mut self) {
        if !self.coherent && !self.flushed {
            self.device.flush_mapped_memory_ranges(&[(self.memory, self.start .. (self.start + self.size as u64))]);
            self.flushed = true;
        }
    }
}

impl<'a, B> Deref for Writer<'a, B>
where
    B: Backend,
{
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        use std::slice::from_raw_parts;
        unsafe {
            from_raw_parts(self.ptr, self.size)
        }
    }
}

impl<'a, B> DerefMut for Writer<'a, B>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut [u8] {
        use std::slice::from_raw_parts_mut;
        self.flushed = false;
        unsafe {
            from_raw_parts_mut(self.ptr, self.size)
        }
    }
}

impl<'a, B> Drop for Writer<'a, B>
where
    B: Backend,
{
    fn drop(&mut self) {
        self.flush();
        self.device.unmap_memory(self.memory);
    }
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
        memory_properties: MemoryProperties,
        arena_size: u64,
        chunk_size: u64,
        min_chunk_size: u64,
    ) -> Self {
        Allocator {
            allocator: SmartAllocator::new(memory_properties, arena_size, chunk_size, min_chunk_size),
            buffer_deletion_queue: DeletionQueue::new(),
            image_deletion_queue: DeletionQueue::new(),
        }
    }

    pub fn create_buffer(
        &mut self,
        device: &B::Device,
        size: u64,
        usage: BufferUsage,
        properties: Properties,
        transient: bool,
    ) -> Result<Buffer<B>, Error> {
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

        let ubuf = device.create_buffer(size, usage)?;
        let requirements = device.get_buffer_requirements(&ubuf);
        let ty = if transient {
            AllocationType::Arena
        } else {
            AllocationType::Chunk
        };
        let block = self.allocator
            .alloc(device, (ty, properties), requirements)
            .map_err(|err| {
                if let Error::NoCompatibleMemoryType = err {
                    Error::BufferUsageAndProperties {
                        usage,
                        properties,
                    }
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
    ) -> Result<Image<B>, Error> {
        let uimg = device.create_image(kind, level, format, usage)?;
        let requirements = device.get_image_requirements(&uimg);
        let ty = AllocationType::Chunk;
        let block = self.allocator
            .alloc(device, (ty, properties), requirements)
            .map_err(|err| {
                if let Error::NoCompatibleMemoryType = err {
                    Error::ImageUsageAndProperties {
                        usage,
                        properties,
                    }
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
