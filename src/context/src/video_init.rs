extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use self::amethyst_renderer::{Renderer, Frame};
use self::amethyst_renderer::target::{ColorFormat, DepthFormat};
use asset_manager::FactoryImpl;
use video_context::{DisplayConfig, VideoContext};

pub fn create_video_context_and_factory_impl(display_config: DisplayConfig) -> (VideoContext, FactoryImpl) {
    match display_config.backend.clone().as_ref() {
        "OpenGL" => new_gl(&display_config),
        #[cfg(windows)]
        "Direct3D" => new_d3d(),
        _ => (VideoContext::Null, FactoryImpl::Null),
    }
}

#[cfg(windows)]
fn new_d3d() -> (VideoContext, FactoryImpl) {
    unimplemented!();
}

fn new_gl(display_config: &DisplayConfig) -> (VideoContext, FactoryImpl) {
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

    let mut frame = Frame::new();
    frame.targets.insert(
        "main".into(),
        Box::new(amethyst_renderer::target::ColorBuffer::new_screen(main_color, main_depth))
    );

    let (w, h) = window.get_inner_size_pixels().unwrap();
    frame.targets.insert(
        "gbuffer".into(),
        Box::new(amethyst_renderer::target::GeometryBuffer::new(&mut factory, (w as u16, h as u16)))
    );

    let video_context = VideoContext::OpenGL {
        window: window,
        device: device,
        renderer: renderer,
        frame: frame,
    };

    let factory_impl = FactoryImpl::OpenGL { factory: factory };
    (video_context, factory_impl)
}
