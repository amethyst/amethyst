
use std::cmp::min;

use gfx_hal::{Backend, Device, Gpu, Instance};
use gfx_hal::adapter::{Adapter, PhysicalDevice};
use gfx_hal::format::{ChannelType, Format, Formatted, Srgba8};
use gfx_hal::queue::{CommandQueue, Compute, General, Graphics, QueueFamily, QueueGroup, QueueType,
                     RawQueueGroup, Transfer};
use gfx_hal::pool::CommandPool;
use gfx_hal::window::{Backbuffer, Surface, SwapchainConfig};

use winit::{EventsLoop, Window, WindowBuilder};


use command::CommandCenter;
use memory::Factory;
use graph::{Graph, Present};
use shaders::{ShaderLoader, ShaderManager};


#[cfg(feature = "metal")]
use metal;


error_chain!{
    errors {
        NoValidAdaptersFound {
            description("No valid adapters queues found")
            display("No valid adapters queues found")
        }
    }
}


pub struct Hal9000<B: Backend> {
    pub device: B::Device,
    pub factory: Factory<B>,
    pub center: CommandCenter<B>,
    pub renderer: Option<Renderer<B>>,
}

pub struct HalBuilder<'a> {
    adapter: Option<&'a str>,
    arena_size: u64,
    chunk_size: u64,
    min_chunk_size: u64,
    compute: bool,
    renderer: Option<RendererBuilder<'a>>,
}


/// Helper trait to initialize backend
pub trait BackendEx: ShaderLoader {
    fn create_window_and_adapters(
        builder: &HalBuilder,
    ) -> Result<(Option<(Window, Self::Surface)>, Vec<Adapter<Self>>)>;
}

#[cfg(feature = "metal")]
impl BackendEx for metal::Backend {
    fn create_window_and_adapters(
        builder: &HalBuilder,
    ) -> Result<(Option<(Window, metal::Surface)>, Vec<Adapter<metal::Backend>>)> {
        let instance = metal::Instance::create("amethyst-hal", 1);

        let window_surface = builder
            .renderer
            .as_ref()
            .map(|renderer| -> Result<_> {
                let window = WindowBuilder::new()
                    .with_dimensions(renderer.width as u32, renderer.height as u32)
                    .with_title(renderer.title)
                    .build(&renderer.events)
                    .chain_err(|| "Failed to create rendering window")?;

                let surface = instance.create_surface(&window);
                Ok(Some((window, surface)))
            })
            .unwrap_or(Ok(None))?;

        Ok((window_surface, instance.enumerate_adapters()))
    }
}


impl<'a> HalBuilder<'a> {
    fn init_adapter<B>(&self, adapter: Adapter<B>) -> (B::Device, Factory<B>, CommandCenter<B>)
    where
        B: Backend,
    {
        println!("Try adapter: {:?}", adapter.info);

        let qf = adapter.queue_families;

        let (transfer, qf) = qf.into_iter().partition::<Vec<_>, _>(
            |qf| qf.queue_type() == QueueType::Transfer,
        );
        let (compute, qf) = qf.into_iter().partition::<Vec<_>, _>(
            |qf| qf.queue_type() == QueueType::Compute,
        );
        let (graphics, qf) = qf.into_iter().partition::<Vec<_>, _>(
            |qf| qf.queue_type() == QueueType::Graphics,
        );
        let (general, _) = qf.into_iter().partition::<Vec<_>, _>(
            |qf| qf.queue_type() == QueueType::General,
        );

        let mut transfer = transfer
            .into_iter()
            .map(|qf| (qf.max_queues(), 0, qf))
            .next();
        let mut compute = compute
            .into_iter()
            .map(|qf| (qf.max_queues(), 0, qf))
            .next();
        let mut graphics = graphics
            .into_iter()
            .map(|qf| (qf.max_queues(), 0, qf))
            .next();
        let mut general = general
            .into_iter()
            .map(|qf| (qf.max_queues(), 0, qf))
            .next();

        if self.compute {
            compute.as_mut().map(|qmr| qmr.1 += 1).or_else(|| {
                general.as_mut().map(|qmr| qmr.1 += 1)
            });
        }

        if self.renderer.is_some() {
            graphics.as_mut().map(|qmr| qmr.1 += 1).or_else(|| {
                general.as_mut().map(|qmr| qmr.1 += 1)
            });
        }

        match (&mut transfer, &mut compute, &mut graphics, &mut general) {
            (&mut Some((_, ref mut req, _)), _, _, _) => *req += 1,
            (_, &mut Some((count, ref mut req, _)), _, _) if count > 1 => *req += 1,
            (_, _, &mut Some((count, ref mut req, _)), _) if count > 1 => *req += 1,
            (_, _, _, &mut Some((count, ref mut req, _))) if count > 2 => *req += 1,
            _ => {}
        };

        let mut requests = vec![];

        {
            let mut push_requests = |qt: Option<(usize, usize, _)>| {
                requests.extend(qt.and_then(|(max, req, qf)| {
                    let count = min(max, req);
                    if count > 0 {
                        Some((qf, vec![1.0; count]))
                    } else {
                        None
                    }
                }));
            };

            push_requests(transfer);
            push_requests(compute);
            push_requests(graphics);
            push_requests(general);
        }

        let Gpu {
            device,
            queue_groups,
            memory_types,
            memory_heaps,
        } = adapter.physical_device.open(requests);
        let factory = Factory::new(
            memory_types,
            self.arena_size,
            self.chunk_size,
            self.min_chunk_size,
        );
        let center = CommandCenter::new(queue_groups);

        (device, factory, center)
    }

    fn build<B>(self) -> Result<Hal9000<B>>
    where
        B: BackendEx,
    {
        let (window_surface, adapters) = B::create_window_and_adapters(&self)?;

        let mut window_surface_format =
            window_surface.map(|(window, surface)| (window, surface, Srgba8::SELF));

        println!("Adapters:");
        for adapter in &adapters {
            println!("\t{:?}", adapter.info);
        }
        let (soft, hard) = adapters.into_iter().partition::<Vec<_>, _>(|adapter| {
            adapter.info.software_rendering
        });
        let (device, factory, center) = hard.into_iter()
            .chain(soft)
            .filter_map(|adapter| {
                if let Some((_, ref surface, ref mut format)) = window_surface_format {
                    *format = find_good_surface_format(surface, &adapter)?;
                }
                Some(self.init_adapter(adapter))
            })
            .next()
            .ok_or(ErrorKind::NoValidAdaptersFound)?;

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
                shaders: ShaderManager::new(),
                graphs: Vec::new(),
            }
        });

        Ok(Hal9000 {
            device,
            factory,
            center,
            renderer,
        })
    }
}


pub struct Renderer<B: Backend> {
    window: Window,
    surface: B::Surface,
    format: Format,
    swapchain: B::Swapchain,
    backbuffer: Backbuffer<B>,
    shaders: ShaderManager<B>,
    graphs: Vec<Graph<B>>,
}

pub struct RendererBuilder<'a> {
    title: &'a str,
    width: u16,
    height: u16,
    events: &'a EventsLoop,
}



fn find_surface_format<B: Backend>(
    surface: &B::Surface,
    adapter: &Adapter<B>,
    channel: ChannelType,
) -> Option<Format> {
    surface
        .capabilities_and_formats(&adapter.physical_device)
        .1
        .into_iter()
        .find(|format| format.1 == channel)
}

fn find_good_surface_format<B: Backend>(
    surface: &B::Surface,
    adapter: &Adapter<B>,
) -> Option<Format> {
    find_surface_format(surface, adapter, ChannelType::Srgb)
        .or_else(|| find_surface_format(surface, adapter, ChannelType::Unorm))
}
