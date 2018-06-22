use config::DisplayConfig;
use error::{Error, Result};
use fnv::FnvHashMap as HashMap;
use gfx::memory::Pod;
use mesh::{Mesh, MeshBuilder, VertexDataSet};
use pipe::{
    ColorBuffer, DepthBuffer, PipelineBuild, PipelineData, PolyPipeline, Target, TargetBuilder,
};
use tex::{Texture, TextureBuilder};
use types::{ColorFormat, DepthFormat, Device, Encoder, Factory, Window};
use winit::{EventsLoop, Window as WinitWindow, WindowBuilder};

/// Generic renderer.
pub struct Renderer {
    /// The gfx factory used for creation of buffers.
    pub factory: Factory,

    device: Device,
    encoder: Encoder,
    main_target: Target,
    window: Window,
    events: EventsLoop,
    multisampling: u16,
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
        pb.build(&mut self.factory, &self.main_target, self.multisampling)
    }

    /// Draws a scene with the given pipeline.
    pub fn draw<'a, P>(&mut self, pipe: &mut P, data: <P as PipelineData<'a>>::Data)
    where
        P: PolyPipeline,
    {
        use gfx::Device;
        #[cfg(feature = "opengl")]
        use glutin::GlContext;

        if let Some(size) = self.window().get_inner_size() {
            if size != self.cached_size {
                self.cached_size = size;
                #[cfg(feature = "opengl")]
                self.window.resize(size.0, size.1);
                self.resize(pipe, size);
            }
        }

        pipe.apply(&mut self.encoder, self.factory.clone(), data);
        self.encoder.flush(&mut self.device);
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
    config: DisplayConfig,
    events: EventsLoop,
    winit_builder: WindowBuilder,
}

impl RendererBuilder {
    /// Creates a new `RendererBuilder`.
    pub fn new(el: EventsLoop) -> Self {
        RendererBuilder {
            config: DisplayConfig::default(),
            events: el,
            winit_builder: WindowBuilder::new().with_title("Amethyst"),
        }
    }

    /// Applies configuration from `Config`
    pub fn with_config(&mut self, config: DisplayConfig) -> &mut Self {
        self.config = config;
        let mut wb = self.winit_builder.clone();
        wb = wb.with_title(self.config.title.clone())
            .with_visibility(self.config.visibility);

        if self.config.fullscreen {
            wb = wb.with_fullscreen(Some(self.events.get_primary_monitor()));
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

    /// Consumes the builder and creates the new `Renderer`.
    pub fn build(self) -> Result<Renderer> {
        let Backend(device, mut factory, main_target, window) =
            init_backend(self.winit_builder.clone(), &self.events, &self.config)?;

        let cached_size = window
            .get_inner_size()
            .expect("Unable to fetch window size, as the window went away!");
        let encoder = factory.create_command_buffer().into();
        Ok(Renderer {
            device,
            encoder,
            factory,
            main_target,
            window,
            events: self.events,
            multisampling: self.config.multisampling,
            cached_size,
        })
    }
}

/// Represents a graphics backend for the renderer.
struct Backend(pub Device, pub Factory, pub Target, pub Window);

/// Creates the Direct3D 11 backend.
#[cfg(all(feature = "d3d11", target_os = "windows"))]
fn init_backend(wb: WindowBuilder, el: &EventsLoop, config: &DisplayConfig) -> Result<Backend> {
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
fn init_backend(wb: WindowBuilder, el: &EventsLoop, config: &DisplayConfig) -> Result<Backend> {
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
fn init_backend(wb: WindowBuilder, el: &EventsLoop, config: &DisplayConfig) -> Result<Backend> {
    use gfx_window_glutin as win;
    use glutin::{self, GlProfile, GlRequest};

    let ctx = glutin::ContextBuilder::new()
        .with_multisampling(config.multisampling)
        .with_vsync(config.vsync)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Latest);

    let (win, dev, fac, color, depth) = win::init::<ColorFormat, DepthFormat>(wb, ctx, el);
    let size = win.get_inner_size().ok_or(Error::WindowDestroyed)?;
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
