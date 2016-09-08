//! This module contains `VideoContext` enum which holds all the resources related to video subsystem.

extern crate amethyst_renderer;
extern crate glutin;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate gfx;

use amethyst_config::Element;
use std::path::Path;
use self::amethyst_renderer::{Renderer, Frame};
use self::amethyst_renderer::target::{ColorFormat, DepthFormat, ColorBuffer};

config!(
    /// Contains display config,
    /// it is required to create a `VideoContext`
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

/// Contains all resources related to video subsystem.
pub struct VideoContext {
    pub window: glutin::Window,
    pub device: gfx_device_gl::Device,
    pub renderer: Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    pub frame: Frame<gfx_device_gl::Resources>,
}

impl VideoContext {

    pub fn new(display_config: &DisplayConfig) -> (VideoContext, gfx_device_gl::Factory) {
        let title = display_config.title.clone();
        let multisampling = display_config.multisampling.clone();
        let visibility = display_config.visibility.clone();

        let mut builder = glutin::WindowBuilder::new()
            .with_title(title)
            .with_multisampling(multisampling)
            .with_visibility(visibility);

        if let Some ((w, h)) = display_config.dimensions {
            builder = builder.with_dimensions(w, h);
        }

        if let Some ((w_min, h_min)) = display_config.min_dimensions {
            builder = builder.with_min_dimensions(w_min, h_min);
        }

        if let Some ((w_max, h_max)) = display_config.max_dimensions {
            builder = builder.with_max_dimensions(w_max, h_max);
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
        let mut renderer = Renderer::new(combuf);
        renderer.load_all(&mut factory);

        let mut frame = Frame::new();
        frame.targets.insert(
            "main".into(),
            Box::new(ColorBuffer{
                color: main_color,
                output_depth: main_depth
            }
            ));

        let (w, h) = window.get_inner_size().unwrap();
        frame.targets.insert(
            "gbuffer".into(),
            Box::new(amethyst_renderer::target::GeometryBuffer::new(&mut factory, (w as u16, h as u16)))
        );

        let video_context = VideoContext {
            window: window,
            device: device,
            renderer: renderer,
            frame: frame,
        };
        (video_context, factory)
    }
}
