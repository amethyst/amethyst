use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;

use hal::{Backend, Instance, Surface};
use hal::buffer;
use hal::format::Format;
use hal::image;
use hal::memory::Properties;
use hal::window::{SurfaceCapabilities, SwapchainConfig};
use mem::{Factory as FactoryTrait, FactoryError, SmartAllocator, Type};
use winit::{Window, WindowId};

use {AmethystGraphBuilder, Buffer, Image};

/// Helper trait that is automatically implemented for all types.
pub trait Same<T>: Into<T> + From<T> + BorrowMut<T> {}
impl<T> Same<T> for T {}

/// Extend backend trait with initialization method.
pub trait BackendEx: Backend {
    type DeviceEx: Same<Self::Device> + Send + Sync;
    type PhysicalDeviceEx: Same<Self::PhysicalDevice> + Send + Sync;
    type SurfaceEx: Same<Self::Surface> + Send + Sync;
    type SwapchainEx: Same<Self::Swapchain> + Send + Sync;
    type Instance: Instance<Backend = Self> + Send + Sync;
    fn init() -> Self::Instance;
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface;
}

#[cfg(feature = "gfx-vulkan")]
impl BackendEx for ::vulkan::Backend {
    type Instance = ::vulkan::Instance;
    fn init() -> Self::Instance {
        ::vulkan::Instance::create("amethyst", 1)
    }
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface {
        instance.create_surface(window)
    }
}

#[cfg(feature = "gfx-metal")]
impl BackendEx for ::metal::Backend {
    type DeviceEx = ::metal::Device;
    type PhysicalDeviceEx = ::metal::PhysicalDevice;
    type SurfaceEx = ::metal::Surface;
    type Instance = ::metal::Instance;
    fn init() -> Self::Instance {
        ::metal::Instance::create("amethyst", 1)
    }
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface {
        instance.create_surface(window)
    }
}

#[cfg(feature = "gfx-dx12")]
impl BackendEx for ::dx12::Backend {
    type Instance = ::dx12::Instance;
    fn init() -> Self::Instance {
        ::dx12::Instance::create("amethyst", 1)
    }
    fn create_surface(instance: &Self::Instance, window: &Window) -> Self::Surface {
        instance.create_surface(window)
    }
}

#[cfg(not(any(feature = "gfx-vulkan", feature = "gfx-metal", feature = "gfx-dx12")))]
impl BackendEx for ::empty::Backend {
    type DeviceEx = ::empty::Device;
    type PhysicalDeviceEx = ::empty::PhysicalDevice;
    type SurfaceEx = ::empty::Surface;
    type SwapchainEx = ::empty::Swapchain;
    type Instance = ::empty::Instance;
    fn init() -> Self::Instance {
        ::empty::Instance
    }
    fn create_surface(_: &Self::Instance, _: &Window) -> Self::Surface {
        ::empty::Surface
    }
}

/// Factory is responsible for creating and destroying `Buffer`s and `Image`s.
#[derive(Derivative)]
// #[derivative(Debug)]
pub struct Factory<B: BackendEx> {
    // #[derivative(Debug="ignore")]
    instance: B::Instance,
    physical: B::PhysicalDeviceEx,
    // #[derivative(Debug="ignore")]
    device: B::DeviceEx,
    allocator: SmartAllocator<B>,
    reclamation: ReclamationQueue<B>,
    current: u64,
    surfaces: Vec<(WindowId, B::SurfaceEx, SwapchainConfig)>,
    graphs: Vec<(WindowId, AmethystGraphBuilder<B>)>,
}

impl<B> Factory<B>
where
    B: BackendEx,
{
    /// Create `Buffer`
    pub fn create_buffer(
        &mut self,
        properties: Properties,
        size: u64,
        usage: buffer::Usage,
    ) -> Result<Buffer<B>, FactoryError> {
        self.allocator.create_buffer(
            self.device.borrow(),
            (Type::General, properties),
            size,
            usage,
        )
    }

    /// Create `Image`
    pub fn create_image(
        &mut self,
        properties: Properties,
        kind: image::Kind,
        level: image::Level,
        format: Format,
        usage: image::Usage,
    ) -> Result<Image<B>, FactoryError> {
        self.allocator.create_image(
            self.device.borrow(),
            (Type::General, properties),
            kind,
            level,
            format,
            usage,
        )
    }

    /// Destroy `Buffer`
    pub fn destroy_buffer(&mut self, buffer: Buffer<B>) {
        self.reclamation.destroy_buffer(self.current, buffer)
    }

    /// Destroy `Image`
    pub fn destroy_image(&mut self, image: Image<B>) {
        self.reclamation.destroy_image(self.current, image)
    }

    /// Add new window to draw upon
    pub fn add_window<F>(&mut self, window: &Window, configure: F)
    where
        F: FnOnce(SurfaceCapabilities, Option<Vec<Format>>) -> SwapchainConfig,
    {
        let surface = B::create_surface(&self.instance, window);
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
    ) -> Self {
        #[allow(dead_code)]
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<Self>();

        Factory {
            instance,
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
        }
    }

    pub(crate) fn device_and_allocator(&mut self) -> (&B::Device, &mut SmartAllocator<B>) {
        (self.device.borrow(), &mut self.allocator)
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

    /// `RenderSystem` calls this once per frame.
    pub(crate) fn advance(&mut self) -> u64 {
        self.current += 1;
        self.current
    }

    /// `RenderSystem` must wait for all operations that was started
    /// before `ongoing`.
    pub(crate) fn clean(&mut self, ongoing: u64) {
        debug_assert!(ongoing <= self.current);
        self.reclamation
            .clear(self.device.borrow(), ongoing, &mut self.allocator)
    }
}

impl<B> Deref for Factory<B>
where
    B: BackendEx,
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
