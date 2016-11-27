extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use gfx_device::DisplayConfig;
use gfx_device::gfx_device_inner::GfxDeviceInner;
use gfx_device::gfx_loader::GfxLoader;

use self::amethyst_renderer::{Renderer, Pipeline};
use self::amethyst_renderer::target::{ColorFormat, DepthFormat, ColorBuffer};

pub fn video_init(display_config: DisplayConfig) -> (GfxDeviceInner, GfxLoader) {
    match display_config.backend.as_str() {
        "OpenGL" => new_gl(&display_config),
        #[cfg(windows)]
        "Direct3D" => new_d3d(),
        _ => (GfxDeviceInner::Null, GfxLoader::Null),
    }
}

#[cfg(windows)]
fn new_d3d() -> (VideoContext, GfxLoader) {
    unimplemented!();
}

fn new_gl(display_config: &DisplayConfig) -> (GfxDeviceInner, GfxLoader) {
    let title = display_config.title.clone();
    let multisampling = display_config.multisampling.clone();
    let visibility = display_config.visibility.clone();

    let mut builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_multisampling(multisampling)
        .with_visibility(visibility);

    if let Some((w, h)) = display_config.dimensions {
        builder = builder.with_dimensions(w, h);
    }

    if let Some((w_min, h_min)) = display_config.min_dimensions {
        builder = builder.with_min_dimensions(w_min, h_min);
    }

    if let Some((w_max, h_max)) = display_config.max_dimensions {
        builder = builder.with_max_dimensions(w_max, h_max);
    }

    if display_config.vsync {
        builder = builder.with_vsync();
    }

    if display_config.fullscreen {
        let monitor = glutin::get_primary_monitor();
        builder = builder.with_fullscreen(monitor);
    }

    let (window, device, mut factory, main_color, main_depth) = gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let combuf = factory.create_command_buffer();
    let mut renderer = Renderer::new(combuf);
    renderer.load_all(&mut factory);

    let mut pipeline = Pipeline::new();
    pipeline.targets.insert("main".into(),
                         Box::new(ColorBuffer {
                             color: main_color,
                             output_depth: main_depth,
                         }));

    let (w, h) = window.get_inner_size().unwrap();
    pipeline.targets.insert("gbuffer".into(),
                         Box::new(amethyst_renderer::target::GeometryBuffer::new(&mut factory, (w as u16, h as u16))));

    let gfx_device_inner = GfxDeviceInner::OpenGL {
        window: window,
        device: device,
        renderer: renderer,
        pipeline: pipeline,
    };

    let gfx_loader = GfxLoader::OpenGL {
        factory: factory,
    };

    (gfx_device_inner, gfx_loader)
}
