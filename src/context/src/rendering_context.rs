extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use amethyst_config::Element;
use std::path::Path;
use self::glutin::{Window, WindowBuilder};
use self::amethyst_renderer::Renderer;
use self::amethyst_renderer::target::{ColorFormat, DepthFormat};
use self::gfx_device_gl::{Device, Factory, Resources, CommandBuffer};
use self::gfx::handle::{RenderTargetView, DepthStencilView};

config!(
    /// Contains display config,
    /// it is required to create a `(Window, RenderingContext)` pair
    struct DisplayConfig {
        pub title: String = "Amethyst game".to_string(),
        pub fullscreen: bool = false,
        pub dimensions: Option<(u32, u32)> = None,
        pub min_dimensions: Option<(u32, u32)> = None,
        pub max_dimensions: Option<(u32, u32)> = None,
        pub vsync: bool = true,
        pub multisampling: u16 = 1,
        pub visibility: bool = true,
    }
);

/// Contains resources required for rendering
pub struct RenderingContext {
    pub device: Device,
    pub factory: Factory,
    pub renderer: Renderer<Resources, CommandBuffer>,
    pub main_color: RenderTargetView<Resources, ColorFormat>,
    pub main_depth: DepthStencilView<Resources, DepthFormat>,
}

impl RenderingContext {
    //! Creates a `(Window, RenderingContext)` pair configured according to `DisplayConfig`
    pub fn new(display_config: DisplayConfig) -> (Window, RenderingContext) {
        let title = display_config.title;
        let multisampling = display_config.multisampling;
        let visibility = display_config.visibility;

        let mut builder = WindowBuilder::new()
            .with_title(title)
            .with_multisampling(multisampling)
            .with_visibility(visibility);

        match display_config.dimensions {
            Some ((w, h)) => builder = builder.with_dimensions(w, h),
            None => (),
        }

        match display_config.min_dimensions {
            Some ((w_min, h_min)) => builder = builder.with_min_dimensions(w_min, h_min),
            None => (),
        }

        match display_config.max_dimensions {
            Some ((w_max, h_max)) => builder = builder.with_max_dimensions(w_max, h_max),
            None => (),
        }

        if display_config.vsync {
            builder = builder.with_vsync();
        }

        if display_config.fullscreen {
            let monitor = glutin::get_primary_monitor();
            builder = builder.with_fullscreen(monitor);
        }

        let (window, device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

        let combuf = factory.create_command_buffer();
        let renderer = Renderer::new(combuf);

        let rendering_context = RenderingContext {
            device: device,
            factory: factory,
            renderer: renderer,
            main_color: main_color,
            main_depth: main_depth,
        };
        (window, rendering_context)
    }
}
