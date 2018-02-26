use std::any::Any;
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::ops::Deref;
use std::slice::from_raw_parts_mut;

use hal::{Backend, Device, Instance, Surface};
use hal::buffer;
use hal::command::{self, RawCommandBuffer};
use hal::device;
use hal::format::Format;
use hal::image;
use hal::mapping::Error as MappingError;
use hal::memory::Properties;
use hal::pool::{self, RawCommandPool};
use hal::queue;
use hal::window::{SurfaceCapabilities, SwapchainConfig};
use mem::{Block, Factory as FactoryTrait, SmartAllocator, SmartBlock, Type};
use winit::{Window, WindowId};

use {AmethystGraphBuilder, Buffer, Image, Error};
use backend::BackendEx;

struct Uploader<B: Backend> {
    staging_treshold: usize,
    family: queue::QueueFamilyId,
    pool: Option<B::CommandPool>,
    cbuf: Option<B::CommandBuffer>,
    free: Vec<B::CommandBuffer>,
    used: VecDeque<(B::CommandBuffer, u64)>,
}

impl<B> Uploader<B>
where
    B: Backend,
{
    fn new(staging_treshold: usize, family: queue::QueueFamilyId) -> Self {
        Uploader {
            staging_treshold,
            family,
            pool: None,
            cbuf: None,
            free: Vec::new(),
            used: VecDeque::new(),
        }
    }

    fn upload_buffer(&mut self, device: &B::Device, allocator: &mut SmartAllocator<B>, buffer: &mut Buffer<B>, offset: u64, data: &[u8]) -> Result<Option<Buffer<B>>, Error> {
        if buffer.size() < offset + data.len() as u64 {
            return Err(Error::with_chain(MappingError::OutOfBounds, "Buffer upload failed"));
        }
        let props = allocator.properties(buffer.block());
        if props.contains(Properties::CPU_VISIBLE) {
            Self::upload_visible_block(device, props.contains(Properties::COHERENT), buffer.block(), offset, data);
            Ok(None)
        } else {
            self.upload_device_local_buffer(device, allocator, buffer, offset, data)
        }
    }

    fn upload_device_local_buffer(&mut self, device: &B::Device, allocator: &mut SmartAllocator<B>, buffer: &mut Buffer<B>, offset: u64, data: &[u8]) -> Result<Option<Buffer<B>>, Error> {
        if data.len() <= self.staging_treshold {
            self.get_command_buffer(device).update_buffer((&*buffer).borrow(), offset, data);
            Ok(None)
        } else {
            let staging = allocator.create_buffer(device, (Type::ShortLived, Properties::CPU_VISIBLE), data.len() as u64, buffer::Usage::TRANSFER_SRC).map_err(|err| Error::with_chain(err, "Failed to create staging buffer"))?;
            let props = allocator.properties(staging.block());
            Self::upload_visible_block(device, props.contains(Properties::COHERENT), staging.block(), 0, data);
            self.get_command_buffer(device).copy_buffer(staging.borrow(), (&*buffer).borrow(), Some(command::BufferCopy {
                src: 0,
                dst: offset,
                size: data.len() as u64,
            }));
            Ok(Some(staging))
        }
    }

    fn upload_image(&mut self, device: &B::Device, allocator: &mut SmartAllocator<B>, image: &mut Image<B>, data: &[u8], layout: image::ImageLayout, upload: ImageUpload) -> Result<Buffer<B>, Error> {
        let staging = allocator.create_buffer(device, (Type::ShortLived, Properties::CPU_VISIBLE), data.len() as u64, buffer::Usage::TRANSFER_SRC).map_err(|err| Error::with_chain(err, "Failed to create staging buffer"))?;
        let props = allocator.properties(staging.block());
        Self::upload_visible_block(device, props.contains(Properties::COHERENT), staging.block(), 0, data);
        self.get_command_buffer(device).copy_buffer_to_image(staging.borrow(), (&*image).borrow(), layout, Some(command::BufferImageCopy {
            buffer_offset: 0,
            buffer_width: 0,
            buffer_height: 0,
            image_layers: upload.layers,
            image_offset: upload.offset,
            image_extent: upload.extent,
        }));
        Ok(staging)
    }

    fn upload_visible_block(device: &B::Device, coherent: bool, block: &SmartBlock<B::Memory>, offset: u64, data: &[u8]) {
        let start = block.range().start + offset;
        let end = start + data.len() as u64;
        let range = start .. end;
        debug_assert!(end <= block.range().end, "Checked in `Uploader::upload` method");
        let ptr = device.map_memory(block.memory(), range.clone()).expect("Expect to be mapped");
        if !coherent {
            device.invalidate_mapped_memory_ranges(Some((block.memory(), range.clone())));
        }
        let slice = unsafe {
            from_raw_parts_mut(ptr, data.len())
        };
        slice.copy_from_slice(data);
        if !coherent {
            device.flush_mapped_memory_ranges(Some((block.memory(), range)));
        }
    }

    fn get_command_buffer<'a>(&'a mut self, device: &B::Device) -> &'a mut B::CommandBuffer {
        let Uploader { family, ref mut pool, ref mut free, ref mut cbuf, .. } = *self;
        cbuf.get_or_insert_with(|| {
            let mut cbuf = free.pop().unwrap_or_else(|| {
                let pool = pool.get_or_insert_with(|| device.create_command_pool(family, pool::CommandPoolCreateFlags::empty()));
                pool.allocate(1, command::RawLevel::Primary).remove(0)
            });
            cbuf.begin(command::CommandBufferFlags::empty());
            cbuf
        })
    }

    fn uploads(&mut self, frame: u64) -> Option<(&mut B::CommandBuffer, queue::QueueFamilyId)> {
        if let Some(mut cbuf) = self.cbuf.take() {
            cbuf.finish();
            self.used.push_back((cbuf, frame));
            Some((&mut self.used.back_mut().unwrap().0, self.family))
        } else {
            None
        }
    }

    fn clear(&mut self, ongoin: u64) {
        while let Some((mut cbuf, frame)) = self.used.pop_front() {
            if frame >= ongoin {
                self.used.push_front((cbuf, ongoin));
                break;
            }
            cbuf.reset(true);
            self.free.push(cbuf);
        }
    }
}

/// Factory is responsible for creating and destroying `Buffer`s and `Image`s.
pub struct Factory<B: Backend> {
    instance: Box<Instance<Backend = B>>,
    physical: B::PhysicalDevice,
    device: B::Device,
    allocator: SmartAllocator<B>,
    reclamation: ReclamationQueue<B>,
    current: u64,
    surfaces: Vec<(WindowId, B::Surface, SwapchainConfig)>,
    graphs: Vec<(WindowId, AmethystGraphBuilder<B>)>,
    uploader: Uploader<B>,
}

impl<B> Factory<B>
where
    B: Backend,
{
    /// Create `Buffer`
    pub fn create_buffer(
        &mut self,
        properties: Properties,
        size: u64,
        usage: buffer::Usage,
    ) -> Result<Buffer<B>, Error> {
        self.allocator.create_buffer(
            self.device.borrow(),
            (Type::General, properties),
            size,
            usage,
        ).map_err(|err| Error::with_chain(err, "Failed to create buffer"))
    }

    /// Create `Image`
    pub fn create_image(
        &mut self,
        properties: Properties,
        kind: image::Kind,
        level: image::Level,
        format: Format,
        usage: image::Usage,
    ) -> Result<Image<B>, Error> {
        self.allocator.create_image(
            self.device.borrow(),
            (Type::General, properties),
            kind,
            level,
            format,
            usage,
        ).map_err(|err| Error::with_chain(err, "Failed to create image"))
    }

    /// Destroy `Buffer`
    pub fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.reclamation.destroy_buffer(self.current, buffer)
    }

    /// Destroy `Image`
    pub fn destroy_image(&mut self, image: Image<B>) {
        self.reclamation.destroy_image(self.current, image)
    }

    /// Upload data to the buffer.
    /// Factory will try to use most appropriate way to write data to the buffer.
    /// For cpu-visible buffers it will write via memory mapping.
    /// If size of the `data` is bigger than `staging_treshold` then it will perform staging.
    /// Otherwise it will write through command buffer directly.
    /// 
    /// # Parameters
    /// `buffer`    - where to upload data. It must be created with at least one of `TRANSFER_DST` usage or `CPU_VISIBLE` property.
    /// `offset`    - write data to the buffer starting from this byte.
    /// `data`      - data to upload.
    /// 
    pub fn upload_buffer(&mut self, buffer: &mut Buffer<B>, offset: u64, data: &[u8]) -> Result<(), Error> {
        let ref device = self.device;
        let ref mut allocator = self.allocator;
        if let Some(staging) = self.uploader.upload_buffer(device, allocator, buffer, offset, data)? {
            self.reclamation.destroy_buffer(self.current, staging);
        }
        Ok(())
    }

    /// Upload data to the image.
    pub fn upload_image(&mut self, image: &mut Image<B>, data: &[u8], layout: image::ImageLayout, upload: ImageUpload) -> Result<(), Error> {
        let ref device = self.device;
        let ref mut allocator = self.allocator;
        let staging = self.uploader.upload_image(device, allocator, image, data, layout, upload)?;
        self.reclamation.destroy_buffer(self.current, staging);
        Ok(())
    }

    /// Add new window to draw upon
    pub fn add_window<F>(&mut self, window: &Window, configure: F)
    where
        F: FnOnce(SurfaceCapabilities, Option<Vec<Format>>) -> SwapchainConfig,
        B: BackendEx,
    {
        let surface = B::create_surface(Any::downcast_ref::<B::Instance>(&self.instance).unwrap(), window);
        let (capabilities, formats) = surface.capabilities_and_formats(self.physical.borrow());
        let config = configure(capabilities, formats);
        self.surfaces.push((window.id(), surface.into(), config));
    }

    /// Add new window to draw upon
    pub fn add_graph(&mut self, window: WindowId, graph: AmethystGraphBuilder<B>) {
        self.graphs.push((window, graph));
    }

    /// Create `Factory` from device and allocator.
    pub(crate) fn new(
        instance: B::Instance,
        physical: B::PhysicalDevice,
        device: B::Device,
        allocator: SmartAllocator<B>,
        staging_treshold: usize,
        upload_family: queue::QueueFamilyId,
    ) -> Self
    where
        B: BackendEx
    {
        #[allow(dead_code)]
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<Self>();

        Factory {
            instance: Box::new(instance),
            physical: physical.into(),
            device: device.into(),
            allocator,
            reclamation: ReclamationQueue {
                offset: 0,
                queue: Vec::new(),
                cache: Vec::new(),
            },
            current: 0,
            surfaces: Vec::new(),
            graphs: Vec::new(),
            uploader: Uploader::new(staging_treshold, upload_family),
        }
    }

    pub(crate) fn device_and_allocator(&mut self) -> (&B::Device, &mut SmartAllocator<B>) {
        (self.device.borrow(), &mut self.allocator)
    }

    pub(crate) fn uploads(&mut self) -> Option<(&mut B::CommandBuffer, queue::QueueFamilyId)> {
        self.uploader.uploads(self.current)
    }

    pub(crate) fn get_surfaces<E>(&mut self, extend: &mut E)
    where
        E: Extend<(WindowId, B::Surface, SwapchainConfig)>,
    {
        extend.extend(self.surfaces.drain(..).map(|(w, s, c)| (w, s.into(), c)));
    }

    pub(crate) fn get_graphs<E>(&mut self, extend: &mut E)
    where
        E: Extend<(WindowId, AmethystGraphBuilder<B>)>,
    {
        extend.extend(self.graphs.drain(..));
    }

    /// `RenderSystem` call this to know with which frame index recorded commands are associated.
    pub(crate) fn current(&mut self) -> u64 {
        self.current
    }

    /// `RenderSystem` call this with least frame index with which ongoing job is associated.
    /// Hence all resources released before this index can be destroyed.
    pub(crate) fn advance(&mut self, ongoing: u64) {
        debug_assert!(ongoing <= self.current);
        self.reclamation
            .clear(self.device.borrow(), ongoing, &mut self.allocator);
        self.uploader.clear(ongoing);
        self.current += 1;
    }
}

impl<B> Deref for Factory<B>
where
    B: Backend,
{
    type Target = B::Device;
    fn deref(&self) -> &B::Device {
        self.device.borrow()
    }
}

#[derive(Debug)]
enum AnyItem<B: Backend> {
    Buffer(Buffer<B>),
    Image(Image<B>),
}

impl<B> AnyItem<B>
where
    B: Backend,
{
    fn destroy(self, device: &B::Device, allocator: &mut SmartAllocator<B>) {
        match self {
            AnyItem::Buffer(buffer) => allocator.destroy_buffer(device, buffer),
            AnyItem::Image(image) => allocator.destroy_image(device, image),
        }
    }
}

#[derive(Debug)]
struct ReclamationNode<B: Backend> {
    items: Vec<AnyItem<B>>,
}

#[derive(Debug)]
struct ReclamationQueue<B: Backend> {
    offset: u64,
    queue: Vec<ReclamationNode<B>>,
    cache: Vec<ReclamationNode<B>>,
}

impl<B> ReclamationQueue<B>
where
    B: Backend,
{
    fn grow(&mut self, current: u64) {
        if self.queue.is_empty() {
            self.offset = current;
        }
        for _ in self.queue.len()..(current - self.offset + 1) as usize {
            self.queue.push(
                self.cache
                    .pop()
                    .unwrap_or_else(|| ReclamationNode { items: Vec::new() }),
            )
        }
    }
    fn destroy_buffer(&mut self, current: u64, buffer: Buffer<B>) {
        self.grow(current);
        self.queue[(current - self.offset) as usize]
            .items
            .push(AnyItem::Buffer(buffer));
    }
    fn destroy_image(&mut self, current: u64, image: Image<B>) {
        self.grow(current);
        self.queue[(current - self.offset) as usize]
            .items
            .push(AnyItem::Image(image));
    }
    fn clear(&mut self, device: &B::Device, ongoing: u64, allocator: &mut SmartAllocator<B>) {
        for mut node in self.queue.drain(..(ongoing - self.offset) as usize) {
            for item in node.items.drain(..) {
                item.destroy(device, allocator);
            }
            self.cache.push(node);
        }
        self.offset = ongoing;
    }
}


pub struct ImageUpload {
    pub layers: image::SubresourceLayers,
    pub offset: command::Offset,
    pub extent: device::Extent,
}
