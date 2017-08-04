//! A data parallel rendering engine developed by the [Amethyst][am] project.
//! The source code is available for download on [GitHub][gh]. See the
//! [online book][bk] for a complete guide to using Amethyst.
//!
//! [am]: https://www.amethyst.rs/
//! [gh]: https://github.com/amethyst/amethyst/tree/develop/src/renderer
//! [bk]: https://www.amethyst.rs/book/
//!
//! # Example
//!
//! ```rust,no_run
//! # extern crate amethyst_renderer;
//! # extern crate winit;
//! #
//! # use amethyst_renderer::{Mesh, Pipeline, Renderer, Result, Scene};
//! # use amethyst_renderer::light::PointLight;
//! # use amethyst_renderer::vertex::PosColor;
//! # use std::time::{Duration, Instant};
//! # use winit::{Event, EventsLoop, Window, WindowEvent};
//! #
//! # fn some_sphere_gen_func() -> &'static [PosColor] {
//! #     &[]
//! # }
//! #
//! # fn run_example() -> Result<()> {
//! let mut events = winit::EventsLoop::new();
//! let mut renderer = Renderer::new(&events)?;
//! let pipe = renderer.create_pipe(Pipeline::deferred())?;
//!
//! let verts = some_sphere_gen_func();
//! let sphere = renderer.create_mesh(Mesh::build(&verts))?;
//!
//! let light = PointLight::default();
//!
//! let mut scene = Scene::default();
//! //scene.add_mesh(sphere)
//! scene.add_light(light);
//!
//! let mut delta = Duration::from_secs(0);
//! let mut running = true;
//! while running {
//!     let start = Instant::now();
//!
//!     events.poll_events(|e| {
//!         match e {
//!             Event::WindowEvent { event, .. } => match event {
//!                 WindowEvent::KeyboardInput { .. } |
//!                 WindowEvent::Closed => running = false,
//!                 _ => (),
//!             },
//!             _ => (),
//!         }
//!     });
//!
//!     renderer.draw(&scene, &pipe, delta);
//!     delta = Instant::now() - start;
//! }
//! # Ok(())
//! # }
//! #
//! # fn main() {
//! #     run_example().unwrap();
//! # }
//! ```

#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate cgmath;
#[macro_use]
extern crate derivative;
extern crate fnv;
extern crate gfx;
extern crate gfx_core;
#[macro_use]
extern crate gfx_macros;
extern crate num_cpus;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate winit;

#[cfg(all(feature = "d3d11", target_os = "windows"))]
extern crate gfx_device_dx11;
#[cfg(all(feature = "d3d11", target_os = "windows"))]
extern crate gfx_window_dxgi;

#[cfg(all(feature = "metal", target_os = "macos"))]
extern crate gfx_device_metal;
#[cfg(all(feature = "metal", target_os = "macos"))]
extern crate gfx_window_metal;

#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

#[cfg(feature = "vulkan")]
extern crate gfx_device_vulkan;
#[cfg(feature = "vulkan")]
extern crate gfx_window_vulkan;

pub use cam::{Camera, Projection};
pub use color::Rgba;
pub use config::Config;
pub use error::{Error, Result};
pub use light::Light;
pub use mesh::{Mesh, MeshBuilder};
pub use mtl::{Material, MaterialBuilder};
pub use pipe::{Pipeline, PipelineBuilder, Stage, Target};
pub use scene::{Model, Scene};
pub use tex::{Texture, TextureBuilder};
pub use vertex::VertexFormat;

use pipe::{ColorBuffer, DepthBuffer};
use rayon::ThreadPool;
use std::sync::Arc;
use std::time::Duration;
use types::{ColorFormat, DepthFormat, Encoder, Factory, Window};
use winit::{EventsLoop, WindowBuilder};

pub mod light;
pub mod pass;
pub mod prelude;
pub mod pipe;
pub mod vertex;

mod cam;
mod color;
mod config;
mod error;
mod mesh;
mod mtl;
mod scene;
mod tex;
mod types;

/// Generic renderer.
pub struct Renderer {
    config: Config,
    device: types::Device,
    encoders: Vec<Encoder>,
    factory: Factory,
    main_target: Arc<Target>,
    pool: Arc<ThreadPool>,
    window: Window,
}

impl Renderer {
    /// Creates a new renderer with default window settings.
    pub fn new(el: &EventsLoop) -> Result<Renderer> {
        RendererBuilder::new(el).build()
    }

    /// Builds a new renderer builder.
    pub fn build(el: &EventsLoop) -> RendererBuilder {
        RendererBuilder::new(el)
    }

    /// Builds a new material resource.
    pub fn create_material(&mut self, mb: MaterialBuilder) -> Result<Material> {
        mb.build(&mut self.factory)
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh(&mut self, mb: MeshBuilder) -> Result<Mesh> {
        mb.build(&mut self.factory)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipe(&mut self, pb: PipelineBuilder) -> Result<Pipeline> {
        pb.build(&mut self.factory, &self.main_target)
    }

    /// Builds a new texture resource.
    pub fn create_texture(&mut self, tb: TextureBuilder) -> Result<Texture> {
        tb.build(&mut self.factory)
    }

    /// Draws a scene with the given pipeline.
    pub fn draw(&mut self, scene: &Scene, pipe: &Pipeline, _delta: Duration) {
        use gfx::Device;
        use glutin::GlContext;
        use rayon::prelude::*;

        let num_threads = self.pool.current_num_threads();
        let encoders_required: usize = pipe.enabled_stages()
            .map(|s| s.encoders_required(num_threads))
            .sum();

        let ref mut fac = self.factory;
        let encoders_count = self.encoders.len();
        if encoders_count < encoders_required {
            self.encoders.extend((encoders_count..encoders_required)
                                 .map(|_| fac.create_command_buffer().into()))
        }

        {
            let mut encoders = self.encoders.as_mut_slice().into_iter();
            let mut updates = Vec::new();
            for stage in pipe.enabled_stages() {
                let needed = stage.encoders_required(num_threads);
                // let enc = {
                //     let slice = encoders;
                //     let (count, left) = slice.split_at_mut(needed);
                //     encoders = left;
                //     count
                // };
                let enc = encoders.by_ref().take(needed);
                updates.push(stage.apply(enc, scene));
            }

            self.pool.install(move || {
                updates.into_par_iter()
                    .flat_map(|u| u)
                    .for_each(|(pass, models, enc)| {
                        for model in models {
                            pass.apply(enc, scene, model);
                        }
                    });
            });
        }
        
        for enc in self.encoders.iter_mut() {
            enc.flush(&mut self.device);
        }

        self.device.cleanup();

        #[cfg(feature = "opengl")]
        self.window.swap_buffers().expect("OpenGL context has been lost");
    }

    /// Returns an immutable reference to the renderer window.
    pub fn window(&self) -> &winit::Window {
        self.window.window()
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use gfx::Device;
        self.device.cleanup();
    }
}

#[allow(missing_docs)]
pub struct RendererBuilder<'a> {
    config: Config,
    events: &'a EventsLoop,
    pool: Option<Arc<ThreadPool>>,
    winit_builder: WindowBuilder,
}

impl<'a> RendererBuilder<'a> {
    #[allow(missing_docs)]
    pub fn new(el: &'a EventsLoop) -> RendererBuilder<'a> {
        RendererBuilder {
            config: Config::default(),
            events: el,
            pool: None,
            winit_builder: WindowBuilder::new().with_title("Amethyst"),
        }
    }

    #[allow(missing_docs)]
    pub fn with_winit_builder(mut self, wb: WindowBuilder) -> Self {
        self.winit_builder = wb;
        self
    }

    #[allow(missing_docs)]
    pub fn with_pool(mut self, pool: Arc<ThreadPool>) -> Self {
        self.pool = Some(pool);
        self
    }

    #[allow(missing_docs)]
    pub fn build(self) -> Result<Renderer> {
        let Backend(dev, fac, main, win) = init_backend(self.winit_builder, self.events)?;

        let num_cores = num_cpus::get();
        let pool = self.pool
            .map(|p| Ok(p))
            .unwrap_or_else(|| {
                                let cfg = rayon::Configuration::new().num_threads(num_cores);
                                ThreadPool::new(cfg)
                                    .map(|p| Arc::new(p))
                                    .map_err(|e| Error::PoolCreation(e))
                            })?;

        Ok(Renderer {
               config: Config::default(),
               device: dev,
               encoders: Vec::new(),
               factory: fac,
               main_target: Arc::new(main),
               pool: pool,
               window: win,
           })
    }
}

/// Represents a graphics backend for the renderer.
struct Backend(pub types::Device, pub Factory, pub Target, pub Window);

/// Creates the Direct3D 11 backend.
#[cfg(all(feature = "d3d11", target_os = "windows"))]
fn init_backend(wb: WindowBuilder, el: &EventsLoop) -> Result<Backend> {
    use gfx_window_dxgi as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(wb, el).unwrap();
    let dev = gfx_device_dx11::Deferred::from(dev);

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target = Target::new(
        ColorBuffer {
            as_input: None,
            as_output: color,
        },
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size
    );

    Ok(Backend(dev, fac, main_target, win))
}

#[cfg(all(feature = "metal", target_os = "macos"))]
fn init_backend(wb: WindowBuilder, el: &EventsLoop) -> Result<Backend> {
    use gfx_window_metal as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(wb, el).unwrap();

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target = Target::new(
        ColorBuffer {
            as_input: None,
            as_output: color,
        },
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size
    );

    Ok(Backend(dev, fac, main_target, win))
}

/// Creates the OpenGL backend.
#[cfg(feature = "opengl")]
fn init_backend(wb: WindowBuilder, el: &EventsLoop) -> Result<Backend> {
    use glutin::{GlProfile, GlRequest};
    use gfx_window_glutin as win;

    let ctx = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Latest);

    let (win, dev, fac, color, depth) = win::init::<ColorFormat, DepthFormat>(wb, ctx, el);
    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let main_target = Target::new(
        ColorBuffer {
            as_input: None,
            as_output: color,
        },
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size
    );

    Ok(Backend(dev, fac, main_target, win))
}
