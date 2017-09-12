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
//! # extern crate glutin;
//! #
//! # use amethyst_renderer::{Mesh, Pipeline, Renderer, Result, Scene};
//! # use amethyst_renderer::light::PointLight;
//! # use amethyst_renderer::vertex::PosColor;
//! # use std::time::{Duration, Instant};
//! #
//! # fn some_sphere_gen_func() -> &'static [PosColor] {
//! #     &[]
//! # }
//! #
//! # fn run_example() -> Result<()> {
//! let mut events = glutin::EventsLoop::new();
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
//!             glutin::Event::WindowEvent { event, .. } => match event {
//!                 glutin::WindowEvent::KeyboardInput { .. } |
//!                 glutin::WindowEvent::Closed => running = false,
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
pub use types::{Encoder, Factory};
pub use vertex::VertexFormat;

pub mod light;
pub mod pass;
pub mod prelude;
pub mod pipe;
pub mod vertex;

use std::sync::Arc;
use std::time::Duration;

use gfx::memory::Pod;
use pipe::{ColorBuffer, DepthBuffer};
use rayon::ThreadPool;
use winit::{EventsLoop, WindowBuilder};

use types::{ColorFormat, DepthFormat, Window};

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
    /// The gfx factory used for creation of buffers.
    pub factory: Factory,

    device: types::Device,
    encoders: Vec<Encoder>,
    main_target: Arc<Target>,
    pool: Arc<ThreadPool>,
    window: Window,
}

impl Renderer {
    /// Creates a `Renderer` with default window settings.
    pub fn new(el: &EventsLoop) -> Result<Renderer> {
        RendererBuilder::new(el).build()
    }
	/// Gets the size of the window, if it still exists
	pub fn window_size(&self)->Option<(u32,u32)>{
		self.window.get_inner_size_pixels()
	}
	/// Set renderer's main target
	pub fn set_main_target(&mut self,target:Target){
		self.main_target = Arc::new(target);
	}
    /// Creates a new `RendererBuilder`, equivalent to `RendererBuilder::new()`.
    pub fn build(el: &EventsLoop) -> RendererBuilder {
        RendererBuilder::new(el)
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh<D, T>(&mut self, mb: MeshBuilder<D, T>) -> Result<Mesh>
    where
        D: AsRef<[T]>,
        T: VertexFormat,
    {
        mb.build(&mut self.factory)
    }

    /// Builds a new texture resource.
    pub fn create_texture<D, T>(&mut self, tb: TextureBuilder<D, T>) -> Result<Texture>
    where
        D: AsRef<[T]>,
        T: Pod,
    {
        tb.build(&mut self.factory)
    }

    /// Builds a new material resource.
    pub fn create_material<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>(
        &mut self,
        mb: MaterialBuilder<DA, TA, DE, TE, DN, TN, DM, TM, DR, TR, DO, TO, DC, TC>,
    ) -> Result<Material>
    where
        DA: AsRef<[TA]>,
        TA: Pod,
        DE: AsRef<[TE]>,
        TE: Pod,
        DN: AsRef<[TN]>,
        TN: Pod,
        DM: AsRef<[TM]>,
        TM: Pod,
        DR: AsRef<[TR]>,
        TR: Pod,
        DO: AsRef<[TO]>,
        TO: Pod,
        DC: AsRef<[TC]>,
        TC: Pod,
    {
        mb.build(&mut self.factory)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipe(&mut self, pb: PipelineBuilder) -> Result<Pipeline> {
        pb.build(&mut self.factory, &self.main_target)
    }
	/// Updates the rendering target (useful for resizes)
	/// Works only on tux for now >_<
	pub fn regen_target(&mut self)->Option<Target>{
        #[cfg(all(feature = "d3d11", target_os = "windows"))]
        use gfx_window_dxgi as win;
        #[cfg(all(feature = "metal", target_os = "macos"))]
        use gfx_window_metal as win;
        #[cfg(feature = "opengl")]
        use gfx_window_glutin as win;

    	let (new_color, new_depth) = win::new_views(&self.window);
		if let Some(window_size) = self.window_size(){
			let target = Target::new(
    		    ColorBuffer {
    		        as_input: None,
    		        as_output: new_color,
    		    },
    		    DepthBuffer {
    		        as_input: None,
    		        as_output: new_depth,
    		    },
    		    window_size,
	    	);
			return Some(target);
		}
		return None;
	}
	/// Same as regen_target, but changes main target to the result of regen_target
	pub fn regen_target_local(&mut self)->Option<Target>{
		if let Some(target) = self.regen_target(){
			let clone = target.clone();
			self.main_target = Arc::new(target);
			return Some(clone);
		}
		return None;
	}
	
    /// Draws a scene with the given pipeline.
    pub fn draw(&mut self, scene: &Scene, pipe: &Pipeline, _delta: Duration) {
        use gfx::Device;
        #[cfg(feature = "opengl")]
        use glutin::GlContext;
        use rayon::prelude::*;

        let num_threads = self.pool.current_num_threads();
        let encoders_required: usize = pipe.enabled_stages()
            .map(|s| s.encoders_required(num_threads))
            .sum();

        let ref mut fac = self.factory;
        let encoders_count = self.encoders.len();
        if encoders_count < encoders_required {
            self.encoders.extend(
                (encoders_count..encoders_required).map(|_| {
                    fac.create_command_buffer().into()
                }),
            )
        }

        {
            let mut encoders = self.encoders.iter_mut();
            self.pool.install(move || {
                let mut updates = Vec::new();
                for stage in pipe.enabled_stages() {
                    let needed = stage.encoders_required(num_threads);
                    let slice = encoders.into_slice();
                    let (taken, left) = slice.split_at_mut(needed);
                    encoders = left.iter_mut();
                    updates.push(stage.apply(taken, scene));
                }

                updates.into_par_iter().flat_map(|update| update).for_each(
                    |(pass, models, enc)| for model in models {
                        pass.apply(enc, scene, model);
                    },
                );
            });
        }

        for enc in self.encoders.iter_mut() {
            enc.flush(&mut self.device);
        }

        self.device.cleanup();

        #[cfg(feature = "opengl")]
        self.window.swap_buffers().expect(
            "OpenGL context has been lost",
        );
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use gfx::Device;
        self.device.cleanup();
    }
}

/// Constructs a new `Renderer`.
pub struct RendererBuilder<'a> {
    config: Config,
    events: &'a EventsLoop,
    pool: Option<Arc<ThreadPool>>,
    winit_builder: WindowBuilder,
}

impl<'a> RendererBuilder<'a> {
    /// Creates a new `RendererBuilder`.
    pub fn new(el: &'a EventsLoop) -> Self {
        RendererBuilder {
            config: Config::default(),
            events: el,
            pool: None,
            winit_builder: WindowBuilder::new().with_title("Amethyst"),
        }
    }

    /// Applies configuration from `Config`
    pub fn with_config(&mut self, config: Config) -> &mut Self {
        self.config = config;
        let mut wb = self.winit_builder.clone();
        wb = wb.with_title(self.config.title.clone()).with_visibility(
            self.config
                .visibility,
        );

        if self.config.fullscreen {
            wb = wb.with_fullscreen(winit::get_primary_monitor());
        }
        match self.config.dimensions {
            Some((width, height)) => {
                wb = wb.with_dimensions(width, height);
            }
            _ => (),
        }
        match self.config.min_dimensions {
            Some((width, height)) => {
                wb = wb.with_min_dimensions(width, height);
            }
            _ => (),
        }
        match self.config.max_dimensions {
            Some((width, height)) => {
                wb = wb.with_max_dimensions(width, height);
            }
            _ => (),
        }
        self.winit_builder = wb;
        self
    }

    /// Applies window settings from the given `glutin::WindowBuilder`.
    pub fn use_winit_builder(&mut self, wb: WindowBuilder) -> &mut Self {
        self.winit_builder = wb;
        self
    }

    /// Specifies an existing thread pool for the `Renderer` to use.
    pub fn with_pool(&mut self, pool: Arc<ThreadPool>) -> &mut Self {
        self.pool = Some(pool);
        self
    }

    /// Consumes the builder and creates the new `Renderer`.
    pub fn build(&self) -> Result<Renderer> {
        let Backend(dev, fac, main, win) =
            init_backend(self.winit_builder.clone(), self.events, &self.config)?;
        let num_cores = num_cpus::get();
        let pool = self.pool.clone().map(|p| Ok(p)).unwrap_or_else(|| {
            let cfg = rayon::Configuration::new().num_threads(num_cores);
            ThreadPool::new(cfg).map(|p| Arc::new(p)).map_err(|e| {
                Error::PoolCreation(format!("{}", e))
            })
        })?;

        Ok(Renderer {
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
fn init_backend(wb: WindowBuilder, el: &EventsLoop, config: &Config) -> Result<Backend> {
    use gfx_window_dxgi as win;

    // FIXME: vsync + multisampling from config
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
        size,
    );

    Ok(Backend(dev, fac, main_target, win))
}

#[cfg(all(feature = "metal", target_os = "macos"))]
fn init_backend(wb: WindowBuilder, el: &EventsLoop, config: &Config) -> Result<Backend> {
    use gfx_window_metal as win;

    // FIXME: vsync + multisampling from config
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
        size,
    );

    Ok(Backend(dev, fac, main_target, win))
}

/// Creates the OpenGL backend.
#[cfg(feature = "opengl")]
fn init_backend(wb: WindowBuilder, el: &EventsLoop, config: &Config) -> Result<Backend> {
    use glutin::{GlProfile, GlRequest};
    use gfx_window_glutin as win;

    let ctx = glutin::ContextBuilder::new()
        .with_multisampling(config.multisampling)
        .with_vsync(config.vsync)
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
        size,
    );

    Ok(Backend(dev, fac, main_target, win))
}

