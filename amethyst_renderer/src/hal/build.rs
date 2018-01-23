use gfx_hal::{Backend, Device, Gpu};
use gfx_hal::adapter::{Adapter, PhysicalDevice};
use gfx_hal::format::{AsFormat, ChannelType, Format};
use gfx_hal::queue::{QueueFamily, QueueType};
use gfx_hal::window::{Surface, SwapchainConfig};

use winit::Window;

use cirque::Cirque;
use command::CommandCenter;
use epoch::{CurrentEpoch, Epoch};
use graph::ColorAttachment;
use hal::Error;
use hal::renderer::{Renderer, RendererConfig};
use memory::Allocator;
use relevant::Relevant;
use upload::Uploader;

#[cfg(feature = "gfx-backend-metal")]
use metal;
#[cfg(feature = "gfx-backend-vulkan")]
use vulkan;

use hal::HalBundle;

pub struct HalConfig<'a> {
    pub adapter: Option<&'a str>,
    pub arena_size: u64,
    pub chunk_size: u64,
    pub min_chunk_size: u64,
    pub compute: bool,
    pub renderer: Option<RendererConfig<'a>>,
}

/// Helper trait to initialize backend
pub trait BackendEx: Backend {
    fn create_window_and_adapters(
        builder: &HalConfig,
    ) -> Result<(Option<(Window, Self::Surface)>, Vec<Adapter<Self>>), ::failure::Error>;
}

#[cfg(feature = "gfx-backend-metal")]
impl BackendEx for metal::Backend {
    fn create_window_and_adapters(
        builder: &HalConfig,
    ) -> Result<
        (
            Option<(Window, metal::Surface)>,
            Vec<Adapter<metal::Backend>>,
        ),
        ::failure::Error,
    > {
        use gfx_hal::Instance;
        use winit::WindowBuilder;

        let instance = metal::Instance::create("amethyst-hal", 1);
        let window_surface = builder
            .renderer
            .as_ref()
            .map(|renderer| -> Result<_, _> {
                let window = WindowBuilder::new()
                    .with_dimensions(renderer.width as u32, renderer.height as u32)
                    .with_title(renderer.title)
                    .with_visibility(true)
                    .build(&renderer.events)
                    .with_context(|_| "Failed to create rendering window")?;

                let surface = instance.create_surface(&window);
                Ok(Some((window, surface)))
            })
            .unwrap_or(Ok(None))?;

        Ok((window_surface, instance.enumerate_adapters()))
    }
}

#[cfg(feature = "gfx-backend-vulkan")]
impl BackendEx for vulkan::Backend {
    fn create_window_and_adapters(
        builder: &HalConfig,
    ) -> Result<
        (
            Option<(Window, <vulkan::Backend as Backend>::Surface)>,
            Vec<Adapter<vulkan::Backend>>,
        ),
        ::failure::Error,
    > {
        use gfx_hal::Instance;
        use winit::WindowBuilder;

        let instance = vulkan::Instance::create("amethyst-hal", 1);
        let window_surface = builder
            .renderer
            .as_ref()
            .map(|renderer| -> Result<_> {
                let window = WindowBuilder::new()
                    .with_dimensions(renderer.width as u32, renderer.height as u32)
                    .with_title(renderer.title)
                    .with_visibility(true)
                    .build(&renderer.events)
                    .with_context(|_| "Failed to create rendering window")?;

                let surface = instance.create_surface(&window);
                Ok(Some((window, surface)))
            })
            .unwrap_or(Ok(None))?;

        Ok((window_surface, instance.enumerate_adapters()))
    }
}

impl<'a> HalConfig<'a> {
    fn init_adapter<B>(&self, adapter: Adapter<B>) -> (B::Device, Allocator<B>, CommandCenter<B>)
    where
        B: Backend,
    {
        println!("Try adapter: {:?}", adapter.info);

        let qf = adapter.queue_families;

        let (transfer, qf) = qf.into_iter()
            .partition::<Vec<_>, _>(|qf| qf.queue_type() == QueueType::Transfer);
        let (compute, qf) = qf.into_iter()
            .partition::<Vec<_>, _>(|qf| qf.queue_type() == QueueType::Compute);
        let (graphics, qf) = qf.into_iter()
            .partition::<Vec<_>, _>(|qf| qf.queue_type() == QueueType::Graphics);
        let (general, _) = qf.into_iter()
            .partition::<Vec<_>, _>(|qf| qf.queue_type() == QueueType::General);

        let transfer = transfer.into_iter().map(|qf| (1, qf)).next();
        let compute = compute.into_iter().map(|qf| (1, qf)).next();
        let graphics = graphics.into_iter().map(|qf| (1, qf)).next();
        let general = general.into_iter().map(|qf| (1, qf)).next();

        let mut requests = vec![];

        let mut graphics_id = None;
        {
            let mut push_requests = |qt: Option<(usize, _)>| {
                requests.extend(qt.and_then(|(count, qf)| {
                    if count > 0 {
                        Some((qf, vec![1.0; count]))
                    } else {
                        None
                    }
                }));
            };

            // push_requests(transfer);
            // push_requests(compute);
            if graphics.is_some() {
                graphics_id = Some(graphics.as_ref().unwrap().1.id());
                push_requests(graphics);
            } else {
                graphics_id = general.as_ref().map(|&(_, ref qf)| qf.id());
                push_requests(general);
            }
        }

        let Gpu {
            device, mut queues, ..
        } = adapter.physical_device.open(requests).unwrap();
        let allocator = Allocator::new(
            adapter.physical_device.memory_properties(),
            self.arena_size,
            self.chunk_size,
            self.min_chunk_size,
        );
        let center = CommandCenter::new(&mut queues, graphics_id);

        (device, allocator, center)
    }

    pub fn build<B>(self) -> Result<HalBundle<B>, ::failure::Error>
    where
        B: BackendEx,
    {
        let (window_surface, adapters) = B::create_window_and_adapters(&self)?;

        let mut window_surface_format =
            window_surface.map(|(window, surface)| (window, surface, Format::Rgb8Srgb));

        println!("Adapters:");
        for adapter in &adapters {
            println!("\t{:?}", adapter.info);
        }
        let (soft, hard) = adapters
            .into_iter()
            .partition::<Vec<_>, _>(|adapter| adapter.info.software_rendering);
        let (device, allocator, center) = hard.into_iter()
            .chain(soft)
            .filter_map(|adapter| {
                if let Some((_, ref surface, ref mut format)) = window_surface_format {
                    *format = find_good_surface_format(surface, &adapter)
                        .ok()?
                        .unwrap_or(Format::Rgb8Srgb);
                }
                Some(self.init_adapter(adapter))
            })
            .next()
            .ok_or(Error::NoValidAdaptersFound)?;

        let renderer = window_surface_format.map(|(window, mut surface, format)| {
            let swapchain_config = SwapchainConfig {
                color_format: format,
                depth_stencil_format: None,
                image_count: 3,
            };

            let (swapchain, backbuffer) = device.create_swapchain(&mut surface, swapchain_config);

            Renderer {
                window,
                surface,
                format,
                swapchain,
                backbuffer,
                graphs: Vec::new(),
                surface_semaphores: Cirque::new(),
                start_epoch: Epoch::new(),
            }
        });

        let uploader = Uploader::new();

        use std::mem::ManuallyDrop;
        Ok(HalBundle {
            relevant: Relevant,
            device,
            allocator,
            center,
            renderer,
            uploader,
            current: CurrentEpoch::new(),
        })
    }
}

fn find_surface_format<B: Backend>(
    surface: &B::Surface,
    adapter: &Adapter<B>,
    channel: ChannelType,
) -> Result<Option<Format>, Error> {
    surface
        .capabilities_and_formats(&adapter.physical_device)
        .1
        .map(|formats| {
            formats
                .into_iter()
                .find(|format| format.base_format().1 == channel)
                .map(Some)
                .ok_or(Error::NoCompatibleFormat.into())
        })
        .unwrap_or(Ok(None))
}

fn find_good_surface_format<B: Backend>(
    surface: &B::Surface,
    adapter: &Adapter<B>,
) -> Result<Option<Format>, Error> {
    find_surface_format(surface, adapter, ChannelType::Srgb)
        .or_else(|_| find_surface_format(surface, adapter, ChannelType::Unorm))
}
