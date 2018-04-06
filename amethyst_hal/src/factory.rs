use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::mem::{forget, ManuallyDrop};
use std::ops::{Deref, Range};
use std::ptr::read;

use hal::{Backend, Instance, Surface};
use hal::buffer;
use hal::format::Format;
use hal::image;
use hal::memory::Properties;
use hal::queue;
use hal::window::{SurfaceCapabilities, SwapchainConfig};

use mem::{Block, Factory as FactoryTrait, SmartAllocator, SmartBlock, Type};

use winit::Window;

use Error;
use backend::BackendEx;
use escape::{Escape, Terminal};
use reclamation::ReclamationQueue;
use uploader::{ImageUpload, Uploader};

pub use mem::Item as RelevantItem;

#[derive(Debug)]
pub struct Item<I, B> {
    inner: ManuallyDrop<RelevantItem<I, B>>,
    escape: Escape<RelevantItem<I, B>>,
}

impl<I, B> Item<I, B> {
    pub fn raw(&self) -> &I {
        self.inner.raw()
    }

    pub fn into_inner(mut self) -> RelevantItem<I, B> {
        let inner = unsafe {
            read(&mut self.escape);
            read(&mut *self.inner)
        };
        forget(self);
        inner
    }
}

pub type RelevantBuffer<B: Backend> = RelevantItem<B::Buffer, SmartBlock<B::Memory>>;
pub type RelevantImage<B: Backend> = RelevantItem<B::Image, SmartBlock<B::Memory>>;

/// Buffer type used in engiene
pub type Buffer<B: Backend> = Item<B::Buffer, SmartBlock<B::Memory>>;

/// Image type used in engiene
pub type Image<B: Backend> = Item<B::Image, SmartBlock<B::Memory>>;

impl<I, B> Borrow<I> for Item<I, B> {
    fn borrow(&self) -> &I {
        (&*self.inner).borrow()
    }
}

impl<I, B> BorrowMut<I> for Item<I, B> {
    fn borrow_mut(&mut self) -> &mut I {
        (&mut *self.inner).borrow_mut()
    }
}

impl<I, B> Block for Item<I, B>
where
    I: Debug + Send + Sync,
    B: Block,
{
    type Memory = B::Memory;
    fn memory(&self) -> &Self::Memory {
        self.inner.memory()
    }
    fn range(&self) -> Range<u64> {
        self.inner.range()
    }
}

impl<I, B> Drop for Item<I, B> {
    fn drop(&mut self) {
        let inner = unsafe { read(&mut *self.inner) };
        self.escape.escape(inner);
    }
}

/// Factory is responsible for creating and destroying `Buffer`s and `Image`s.
pub struct Factory<B: Backend> {
    instance: Box<Instance<Backend = B>>,
    physical: B::PhysicalDevice,
    device: B::Device,
    allocator: SmartAllocator<B>,
    reclamation: ReclamationQueue<AnyItem<B>>,
    current: u64,
    uploader: Uploader<B>,
    buffers: Terminal<RelevantBuffer<B>>,
    images: Terminal<RelevantImage<B>>,
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
        let buffer: RelevantBuffer<B> = self.allocator
            .create_buffer(
                self.device.borrow(),
                (Type::General, properties),
                size,
                usage,
            )
            .map_err(|err| Error::with_chain(err, "Failed to create buffer"))?;
        Ok(Item {
            inner: ManuallyDrop::new(buffer),
            escape: self.buffers.escape(),
        })
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
        let image = self.allocator
            .create_image(
                self.device.borrow(),
                (Type::General, properties),
                kind,
                level,
                format,
                usage,
            )
            .map_err(|err| Error::with_chain(err, "Failed to create image"))?;
        Ok(Item {
            inner: ManuallyDrop::new(image),
            escape: self.images.escape(),
        })
    }

    /// Destroy `Buffer`
    pub fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.reclamation
            .push(self.current, AnyItem::Buffer(buffer.into_inner()));
    }

    /// Destroy `Image`
    pub fn destroy_image(&mut self, image: Image<B>) {
        self.reclamation
            .push(self.current, AnyItem::Image(image.into_inner()));
    }

    /// Destroy `RelevantBuffer`
    pub fn destroy_relevant_buffer(&mut self, buffer: RelevantBuffer<B>) {
        self.reclamation.push(self.current, AnyItem::Buffer(buffer));
    }

    /// Destroy `RelevantImage`
    pub fn destroy_relevant_image(&mut self, image: RelevantImage<B>) {
        self.reclamation.push(self.current, AnyItem::Image(image));
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
    pub fn upload_buffer(
        &mut self,
        buffer: &mut Buffer<B>,
        offset: u64,
        data: &[u8],
    ) -> Result<(), Error> {
        let ref device = self.device;
        let ref mut allocator = self.allocator;
        if let Some(staging) =
            self.uploader
                .upload_buffer(device, allocator, &mut *buffer.inner, offset, data)?
        {
            self.reclamation
                .push(self.current, AnyItem::Buffer(staging));
        }
        Ok(())
    }

    /// Upload data to the image.
    pub fn upload_image(
        &mut self,
        image: &mut Image<B>,
        data: &[u8],
        layout: image::ImageLayout,
        upload: ImageUpload,
    ) -> Result<(), Error> {
        let ref device = self.device;
        let ref mut allocator = self.allocator;
        let staging =
            self.uploader
                .upload_image(device, allocator, &mut *image.inner, data, layout, upload)?;
        self.reclamation
            .push(self.current, AnyItem::Buffer(staging));
        Ok(())
    }

    /// Add new window to draw upon
    pub fn create_surface(&mut self, window: &Window) -> B::Surface
    where
        B: BackendEx,
    {
        B::create_surface(
            Any::downcast_ref::<B::Instance>(&self.instance).unwrap(),
            window,
        )
    }

    /// Get capabilities and formats for surface
    pub fn capabilities_and_formats(
        &self,
        surface: &B::Surface,
    ) -> (SurfaceCapabilities, Option<Vec<Format>>) {
        surface.capabilities_and_formats(&self.physical)
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
        B: BackendEx,
    {
        Factory {
            instance: Box::new(instance),
            physical: physical.into(),
            device: device.into(),
            allocator,
            reclamation: ReclamationQueue::new(),
            current: 0,
            uploader: Uploader::new(staging_treshold, upload_family),
            buffers: Terminal::new(),
            images: Terminal::new(),
        }
    }

    pub(crate) fn device_and_allocator(&mut self) -> (&B::Device, &mut SmartAllocator<B>) {
        (self.device.borrow(), &mut self.allocator)
    }

    pub(crate) fn uploads(&mut self) -> Option<(&mut B::CommandBuffer, queue::QueueFamilyId)> {
        self.uploader.uploads(self.current)
    }

    /// `RenderSystem` call this to know with which frame index recorded commands are associated.
    pub(crate) fn current(&mut self) -> u64 {
        self.current
    }

    /// `RenderSystem` call this with least frame index with which ongoing job is associated.
    /// Hence all resources released before this index can be destroyed.
    pub(crate) unsafe fn advance(&mut self, ongoing: u64) {
        debug_assert!(ongoing <= self.current);
        for buffer in self.buffers.drain() {
            self.reclamation.push(self.current, AnyItem::Buffer(buffer));
        }
        for image in self.images.drain() {
            self.reclamation.push(self.current, AnyItem::Image(image));
        }
        let ref device = self.device;
        let ref mut allocator = self.allocator;
        self.reclamation.clear(ongoing, |item| {
            item.destroy(device, allocator);
        });
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
    Buffer(RelevantBuffer<B>),
    Image(RelevantImage<B>),
}

impl<B> AnyItem<B>
where
    B: Backend,
{
    pub fn destroy(self, device: &B::Device, allocator: &mut SmartAllocator<B>) {
        match self {
            AnyItem::Buffer(buffer) => {
                allocator.destroy_buffer(device, buffer);
            }
            AnyItem::Image(image) => {
                allocator.destroy_image(device, image);
            }
        }
    }
}

#[test]
#[allow(dead_code)]
fn factory_send_sync() {
    fn is_send_sync<T: Send + Sync>() {}
    fn for_any_backend<B: Backend>() {
        is_send_sync::<Factory<B>>();
    }
}
