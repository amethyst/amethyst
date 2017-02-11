extern crate gfx_window_glutin;
extern crate glutin;

use gfx_device::DisplayConfig;
use gfx_device::gfx_device::GfxDevice;
use renderer::target::{ColorFormat, DepthFormat};
use renderer::Renderer;
use gfx_device::gfx_types::{Factory, Encoder};


/// Create a `(GfxDevice, Factory, MainTarget)` tuple from `DisplayConfig`
pub fn video_init(cfg: DisplayConfig, num_cpus: usize) -> (GfxDevice, Factory) {
    #[cfg(feature="opengl")]
    return new_gl(&cfg, num_cpus);
    #[cfg(all(windows, feature="direct3d"))]
    return new_d3d(&cfg, num_cpus);
}

#[cfg(all(windows, feature="direct3d"))]
fn new_d3d(_: &DisplayConfig, _: usize) -> (GfxDevice, Factory) {
    unimplemented!();
}


/// Create a `(GfxDevice, Factory)` tuple from `DisplayConfig`
#[cfg(feature="opengl")]
pub fn new_gl(cfg: &DisplayConfig, num_cpus: usize) -> (GfxDevice, Factory) {
    let title = cfg.title.clone();
    let multisampling = cfg.multisampling.clone();
    let visibility = cfg.visibility.clone();

    let mut builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_multisampling(multisampling)
        .with_visibility(visibility);

    if let Some((w, h)) = cfg.dimensions {
        builder = builder.with_dimensions(w, h);
    }

    if let Some((w_min, h_min)) = cfg.min_dimensions {
        builder = builder.with_min_dimensions(w_min, h_min);
    }

    if let Some((w_max, h_max)) = cfg.max_dimensions {
        builder = builder.with_max_dimensions(w_max, h_max);
    }

    if cfg.vsync {
        builder = builder.with_vsync();
    }

    if cfg.fullscreen {
        let monitor = glutin::get_primary_monitor();
        builder = builder.with_fullscreen(monitor);
    }

    let (window, device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    let mut renderer = Renderer::new();
    renderer.load_all(&mut factory);

    let mut encoders: Vec<Encoder> = vec![];
    for _ in 0..num_cpus {
        let combuf = factory.create_command_buffer();
        let encoder: Encoder = combuf.into();
        encoders.push(encoder);
    }

    let gfx_device = GfxDevice {
        window: window,
        device: device,
        main_color: main_color,
        main_depth: main_depth,
        renderer: renderer,
        encoders: encoders,
    };

    (gfx_device, factory)
}
