
use gfx_hal::{Backend, Gpu, Instance};
use gfx_hal::format::Format;
use winit::{EventsLoop, Window, WindowBuilder};

#[cfg(feature = "metal")]
use metal;


error_chain!{

}

struct Renderer<B: Backend, I, AsRef> {
    window: Window,
    instance: I,
    adapter: Adapter<B>,
    surface: B::Surface,
    surface_format: Format,
    swapchain: B::Swapchain,
    allocator: A,
}

struct RendererBuilder<'a> {
    title: &'a str,
    width: u16,
    height: u16,
}

impl RendererBuilder {
    #[cfg(feature = "metal")]
    fn build(self, events: &EventsLoop) -> Result<Renderer<metal::Backend, metal::Instance>> {
        let window = WindowBuilder::new()
            .with_dimensions(self.width, self.height)
            .with_title(self.title)
            .build(events)
            .chain_err(|| "Failed to create rendering window")?;

        let window_size = window.get_inner_size_pixels().ok_or("Window suddenly vanished")?;
        let pixel_width = window_size.0;
        let pixel_height = window_size.1;

        let instance = metal::Instance::create("amethyst-hal", 1);
        let surface = instance.create_surface(&window);
        let adapters = instance.enumerate_adapters();

        println!("Adapters:");
        for adapter in &adapters {
            println!("\t{:?}", adapter.info);
        }

        let (soft, hard): (Vec<_>, _) = adapters.into_iter().partition(|adapter| adapter.info.software_rendering);
        let adapter = if hard.is_empty() {
            println!("No hardware adapters found")
            soft[0]
        } else {
            hard[0]
        };

        println!("Picked adapter: {:?}", adapter.info);

        let surface_format = surface
            .capabilities_and_formats(&adapter.physical_device)
            .1
            .into_iter()
            .find(|format| format.1 == ChannelType::Srgb)
            .map(Ok)
            .unwrap_or_else(|| {
                surface
                    .capabilities_and_formats(&adapter.physical_device)
                    .1
                    .into_iter()
                    .find(|format| format.1 == ChannelType::Unorm)
                    .ok_or("Can't find appropriate surface format")
            })?;

        let Gpu { device, mut queue_groups, memory_types, .. } =
            adapter.open_with(|family| {
                if family.supports_graphics() && surface.supports_queue_family(family) {
                    Some(1)
                } else {
                    None
                }
            });
    }
}