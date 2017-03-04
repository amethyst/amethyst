use gfx_device::DisplayConfig;
use gfx_device::gfx_device::GfxDevice;
use gfx_device::gfx_types::Factory;
use gfx_device::main_target::MainTarget;
use renderer::Renderer;
use renderer::target::{ColorFormat, DepthFormat};

/// Create a `(GfxDevice, Factory, MainTarget)` tuple from `DisplayConfig`
pub fn video_init(cfg: &DisplayConfig) -> (GfxDevice, Factory, MainTarget) {
    #[cfg(feature="opengl")]
    return new_gl(cfg);
    #[cfg(all(windows, feature="direct3d"))]
    return new_d3d(cfg);
}

#[cfg(all(windows, feature="direct3d"))]
fn new_d3d(_: &DisplayConfig) -> (GfxDevice, Factory, MainTarget) {
    unimplemented!();
}

#[cfg(feature="opengl")]
fn new_gl(cfg: &DisplayConfig) -> (GfxDevice, Factory, MainTarget) {
    use gfx_window_glutin;
    use glutin;

    let title = cfg.title.clone();
    let multisampling = cfg.multisampling;
    let visibility = cfg.visibility;

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

    let combuf = factory.create_command_buffer();
    let mut renderer = Renderer::new(combuf);
    renderer.load_all(&mut factory);

    let gfx_device = GfxDevice {
        window: window,
        device: device,
        renderer: renderer,
    };

    let main_target = MainTarget {
        color: main_color,
        depth: main_depth,
    };

    (gfx_device, factory, main_target)
}
