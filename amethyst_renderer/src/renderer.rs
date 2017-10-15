

use std::sync::Arc;
use std::time::Duration;

use gfx::memory::Pod;
use num_cpus;
use rayon::{self, ThreadPool};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use config::Config;
use error::{Error, Result};
use mesh::{Mesh, MeshBuilder, VertexDataSet};
use pipe::{ColorBuffer, DepthBuffer, PipelineBuild, PipelineData, PolyPipeline, Target,
           TargetBuilder};
use tex::{Texture, TextureBuilder};
use types::{ColorFormat, DepthFormat, Device, Encoder, Factory, Window};
use fnv::FnvHashMap as HashMap;
use winit::{self, EventsLoop, Window as WinitWindow, WindowBuilder};

/// Generic renderer.
pub struct Renderer {
    /// The gfx factory used for creation of buffers.
    pub factory: Factory,

    device: Device,
    encoders: Vec<Encoder>,
    main_target: Target,
    pool: Arc<ThreadPool>,
    window: Window,
    events: EventsLoop,
    cached_size: (u32, u32),
}

impl Renderer {
    /// Creates a `Renderer` with default window settings.
    pub fn new() -> Result<Renderer> {
        Self::build().build()
    }

    /// Creates a new `RendererBuilder`, equivalent to `RendererBuilder::new()`.
    pub fn build_with_loop(el: EventsLoop) -> RendererBuilder {
        RendererBuilder::new(el)
    }

    /// Creates a new `RendererBuilder`, equivalent to `RendererBuilder::new()`.
    pub fn build() -> RendererBuilder {
        Self::build_with_loop(EventsLoop::new())
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh<T>(&mut self, mb: MeshBuilder<T>) -> Result<Mesh>
    where
        T: VertexDataSet,
    {
        mb.build(&mut self.factory)
    }

    /// Builds a new texture resource.
    pub fn create_texture<D, T>(&mut self, tb: TextureBuilder<D, T>) -> Result<Texture>
    where
        D: AsRef<[T]>,
        T: Pod + Copy,
    {
        tb.build(&mut self.factory)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipe<B, P>(&mut self, pb: B) -> Result<P>
    where
        P: PolyPipeline,
        B: PipelineBuild<Pipeline = P>,
    {
        pb.build(&mut self.factory, &self.main_target)
    }

    /// Draws a scene with the given pipeline.
    pub fn draw<'a, P>(
        &mut self,
        pipe: &mut P,
        data: <P as PipelineData<'a>>::Data,
        _delta: Duration,
    ) where
        P: PolyPipeline,
    {
        use gfx::Device;
        #[cfg(feature = "opengl")]
        use glutin::GlContext;

        if let Some(size) = self.window().get_inner_size_pixels() {
            if size != self.cached_size {
                self.cached_size = size;
                self.resize(pipe, size);
            }
        }

        let num_threads = self.pool.current_num_threads();
        let encoders_required = P::encoders_required(num_threads);

        let ref mut fac = self.factory;
        let encoders_count = self.encoders.len();
        if encoders_count < encoders_required {
            self.encoders.extend(
                (encoders_count..encoders_required).map(|_| fac.create_command_buffer().into()),
            )
        }

        {
            let mut encoders = self.encoders.as_mut();
            self.pool.install(move || {
                PolyPipeline::apply(pipe, encoders, num_threads, data)
                    .into_par_iter()
                    .for_each(|()| {});
            });
        }

        for enc in self.encoders.iter_mut() {
            enc.flush(&mut self.device);
        }

        self.device.cleanup();

        #[cfg(feature = "opengl")]
        self.window
            .swap_buffers()
            .expect("OpenGL context has been lost");
    }

    /// Retrieve a mutable borrow of the events loop
    pub fn events_mut(&mut self) -> &mut EventsLoop {
        &mut self.events
    }

    /// Resize the targets associated with this renderer and pipeline.
    pub fn resize<P: PolyPipeline>(&mut self, pipe: &mut P, new_size: (u32, u32)) {
        self.main_target.resize_main_target(&self.window);
        let mut targets = HashMap::default();
        targets.insert("".to_string(), self.main_target.clone());
        for (key, value) in pipe.targets().iter().filter(|&(k, _)| !k.is_empty()) {
            let (key, target) = TargetBuilder::new(key.clone())
                .with_num_color_bufs(value.color_bufs().len())
                .with_depth_buf(value.depth_buf().is_some())
                .build(&mut self.factory, new_size)
                .unwrap();
            targets.insert(key, target);
        }
        pipe.new_targets(targets);
    }

    /// Retrieves an immutable borrow of the window.
    ///
    /// No operations require a mutable borrow as of 2017-10-02
    #[cfg(feature = "opengl")]
    pub fn window(&self) -> &WinitWindow {
        self.window.window()
    }

    #[cfg(feature = "metal")]
    #[cfg(feature = "vulkan")]
    pub fn window(&self) -> &WinitWindow {
        &self.window.0
    }

    #[cfg(feature = "d3d11")]
    pub fn window(&self) -> &WinitWindow {
        &*self.window.0
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use gfx::Device;
        self.device.cleanup();
    }
}

/// Constructs a new `Renderer`.
pub struct RendererBuilder {
    config: Config,
    events: EventsLoop,
    pool: Option<Arc<ThreadPool>>,
    winit_builder: WindowBuilder,
}

impl RendererBuilder {
    /// Creates a new `RendererBuilder`.
    pub fn new(el: EventsLoop) -> Self {
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
        wb = wb.with_title(self.config.title.clone())
            .with_visibility(self.config.visibility);

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
    pub fn build(self) -> Result<Renderer> {
        let Backend(dev, fac, main, win) =
            init_backend(self.winit_builder.clone(), &self.events, &self.config)?;
        let num_cores = num_cpus::get();
        let pool = self.pool.clone().map(|p| Ok(p)).unwrap_or_else(|| {
            let cfg = rayon::Configuration::new().num_threads(num_cores);
            ThreadPool::new(cfg)
                .map(|p| Arc::new(p))
                .map_err(|e| Error::PoolCreation(format!("{}", e)))
        })?;

        let size = win.get_inner_size_pixels()
            .expect("Unable to fetch window size, as the window went away!");

        Ok(Renderer {
            device: dev,
            encoders: Vec::new(),
            factory: fac,
            main_target: main,
            pool: pool,
            window: win,
            events: self.events,
            cached_size: size,
        })
    }
}

/// Represents a graphics backend for the renderer.
struct Backend(pub Device, pub Factory, pub Target, pub Window);

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
    use glutin::{self, GlProfile, GlRequest};
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
