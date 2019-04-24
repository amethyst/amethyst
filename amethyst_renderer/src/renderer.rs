use amethyst_error::{format_err, Error};
use fnv::FnvHashMap as HashMap;
use gfx::memory::Pod;
use winit::{dpi::LogicalSize, EventsLoop, Window as WinitWindow, WindowBuilder};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::{
    config::DisplayConfig,
    error,
    mesh::{Mesh, MeshBuilder, VertexDataSet},
    pipe::{
        ColorBuffer, DepthBuffer, PipelineBuild, PipelineData, PolyPipeline, Target, TargetBuilder,
    },
    tex::{Texture, TextureBuilder},
    types::{ColorFormat, DepthFormat, Device, Encoder, Factory, Window},
};

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
    cached_size: LogicalSize,
    cached_hidpi_factor: f64,
}

impl Renderer {
    /// Creates a `Renderer` with default window settings.
    pub fn new() -> Result<Self, Error> {
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
    pub fn create_mesh<T>(&mut self, mb: MeshBuilder<T>) -> Result<Mesh, Error>
    where
        T: VertexDataSet,
    {
        mb.build(&mut self.factory)
    }

    /// Builds a new texture resource.
    pub fn create_texture<D, T>(&mut self, tb: TextureBuilder<D, T>) -> Result<Texture, Error>
    where
        D: AsRef<[T]>,
        T: Pod + Copy,
    {
        tb.build(&mut self.factory)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipe<B, P>(&mut self, pb: B) -> Result<P, Error>
    where
        P: PolyPipeline,
        B: PipelineBuild<Pipeline = P>,
    {
        pb.build(&mut self.factory, &self.main_target, self.multisampling)
    }

    /// Draws a scene with the given pipeline.
    #[allow(clippy::float_cmp)] // cmp just used to recognize change
    pub fn draw<'a, P>(&mut self, pipe: &mut P, data: <P as PipelineData<'a>>::Data)
    where
        P: PolyPipeline,
    {
        use gfx::Device;
        #[cfg(feature = "opengl")]
        use glutin::dpi::PhysicalSize;

        if let Some(size) = self.window().get_inner_size() {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_draw_size");
            let hidpi_factor = self.window().get_hidpi_factor();

            if size != self.cached_size || hidpi_factor != self.cached_hidpi_factor {
                self.cached_size = size;
                self.cached_hidpi_factor = hidpi_factor;
                #[cfg(feature = "opengl")]
                self.window
                    .resize(PhysicalSize::from_logical(size, hidpi_factor));
                self.resize(pipe, size.into());
            }
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_draw_pipeapply");
            pipe.apply(&mut self.encoder, self.factory.clone(), data);
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_draw_encoderflush");
            self.encoder.flush(&mut self.device);
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_draw_devicecleanup");
            self.device.cleanup();
        }
        {
            #[cfg(feature = "profiler")]
            profile_scope!("render_system_draw_swapbuffers");
            #[cfg(feature = "opengl")]
            self.window
                .swap_buffers()
                .expect("OpenGL context has been lost");
        }
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
                .expect("Unable to create new target when resizing");
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
    window_builder: WindowBuilder,
}

impl RendererBuilder {
    /// Creates a new `RendererBuilder`.
    pub fn new(el: EventsLoop) -> Self {
        let config = DisplayConfig::default();
        RendererBuilder {
            config: config.clone(),
            window_builder: config.to_windowbuilder(el.get_primary_monitor()),
            events: el,
        }
    }

    /// Applies configuration from the provided `Config`.
    pub fn with_config(&mut self, config: DisplayConfig) -> &mut Self {
        self.config = config.clone();
        self.window_builder = config.to_windowbuilder(self.events.get_primary_monitor());
        self
    }

    /// Applies configuraiton from the provided `WindowBuilder`.
    pub fn with_window_builder(&mut self, wb: WindowBuilder) -> &mut Self {
        self.window_builder = wb.clone();
        self.config = wb.into();
        self
    }

    /// Consumes the builder and creates the new `Renderer`.
    pub fn build(self) -> Result<Renderer, Error> {
        let Backend(device, mut factory, main_target, window) =
            init_backend(self.window_builder, &self.events, &self.config)?;

        let cached_size = window
            .get_inner_size()
            .ok_or_else(|| format_err!("Unable to fetch window size, as the window went away."))?;

        if self.config.centered_window && !self.config.fullscreen {
            let windowsize = window.get_outer_size().ok_or_else(|| {
                format_err!("Unable to fetch window outer size, as the window went away.")
            })?;
            let screen_size = window.get_current_monitor().get_dimensions();
            window.set_position(
                (
                    ((screen_size.width * 0.5) - windowsize.width * 0.5) as f64,
                    ((screen_size.height * 0.5) - windowsize.height * 0.5) as f64,
                )
                    .into(),
            );
        }

        let cached_hidpi_factor = window.get_hidpi_factor();

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
            cached_hidpi_factor,
        })
    }
}

/// Represents a graphics backend for the renderer.
struct Backend(pub Device, pub Factory, pub Target, pub Window);

/// Creates the Direct3D 11 backend.
#[cfg(all(feature = "d3d11", target_os = "windows"))]
fn init_backend(
    wb: WindowBuilder,
    el: &EventsLoop,
    config: &DisplayConfig,
) -> Result<Backend, Error> {
    // FIXME: vsync + multisampling from config
    let (win, dev, mut fac, color) = gfx_window_dxgi::init::<ColorFormat>(wb, el)
        .expect("Unable to initialize window (d3d11 backend)");
    let dev = gfx_device_dx11::Deferred::from(dev);

    let size = win
        .get_inner_size_points()
        .ok_or(error::Error::WindowDestroyed)?;
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
fn init_backend(
    wb: WindowBuilder,
    el: &EventsLoop,
    config: &DisplayConfig,
) -> Result<Backend, Error> {
    // FIXME: vsync + multisampling from config
    let (win, dev, mut fac, color) = gfx_window_metal::init::<ColorFormat>(wb, el)
        .expect("Unable to initialize window (metal backend)");

    let size = win
        .get_inner_size_points()
        .ok_or(error::Error::WindowDestroyed)?;
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
fn init_backend(
    wb: WindowBuilder,
    el: &EventsLoop,
    config: &DisplayConfig,
) -> Result<Backend, Error> {
    #[cfg(target_os = "macos")]
    use glutin::{GlProfile, GlRequest};

    let ctx = glutin::ContextBuilder::new()
        .with_multisampling(config.multisampling)
        .with_vsync(config.vsync);
    #[cfg(target_os = "macos")]
    let ctx = ctx
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Latest);

    let (win, dev, fac, color, depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(wb, ctx, el);
    let size = win
        .get_inner_size()
        .ok_or(error::Error::WindowDestroyed)?
        .into();
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
