//! A highly parallel rendering engine developed for the Amethyst game engine.
//!
//! # Example
//!
//! ```ignore
//! let wb = winit::WindowBuilder::new()
//!     .with_title("Amethyst Renderer Demo")
//!     .with_dimensions(800, 600);
//!
//! let (win, mut renderer) = RendererBuilder::new(wb)
//!    .build()
//!    .expect("Could not build renderer");
//!
//! let verts = some_sphere_gen_func();
//! let sphere = renderer.create_mesh(verts)
//!     .with_ambient_texture(Rgba(1.0, 0.5, 0.2, 1.0))
//!     .with_diffuse_texture(Rgba(0.7, 0.3, 0.1, 1.0))
//!     .build()
//!     .expect("Could not build sphere");
//!
//! let light = PointLight::new()
//!     .position(Point3::new(4.0, 6.0, -4.0))
//!     .color(Rgba(1.0, 0.5, 0.2, 1.0))
//!     .radius(1.0)
//!     .intensity(0.7)
//!     .smoothness(0.3);
//!
//! let scene = Scene::new()
//!     .add_object(sphere)
//!     .add_light(light);
//!
//! let pipe = renderer.create_pipeline()
//!     .with_target(Target::new("gbuffer")
//!         .with_num_color_bufs(4)
//!         .with_depth_buf(true))
//!     .with_layer(Layer::with_target("gbuffer")
//!         .with_pass(ClearTarget::with_values([0.0; 4], 0.0))
//!         .with_pass(DrawFlat::with_camera("main_camera")))
//!     .with_layer(Layer::with_target("main")
//!         .with_pass(BlitLayer::from_target_color_buf("gbuffer", 2))
//!         .with_pass(DeferredLighting::new("main_camera", "gbuffer", "scene")))
//!     .build()
//!     .expect("Could not build pipeline");
//!
//! 'main: loop {
//!     for e in win.poll_events() {
//!         match e {
//!             winit::Event::Closed => break 'main,
//!             _ => (),
//!         }
//!     }
//!
//!     renderer.draw(&scene, &pipe);
//!     win.swap_buffers().expect("Could not swap buffers");
//! }
//! ```

#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate cgmath;
extern crate fnv;
#[macro_use]
extern crate gfx;
extern crate num_cpus;
extern crate rayon;
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

pub use error::{Error, Result};
pub use mesh::{Mesh, MeshBuilder};
pub use pass::Pass;
pub use pipe::{Pipeline, PipelineBuilder};
pub use scene::Scene;
pub use stage::{Stage, StageBuilder};
pub use target::{Target, TargetBuilder};
pub use types::VertexFormat;

use std::time::Duration;
use types::{Buffer, ColorFormat, DepthFormat, Encoder, Factory, Resources, Slice, Window};

pub mod mesh;
pub mod pass;
pub mod target;

mod error;
mod pipe;
mod scene;
mod stage;
mod types;

/// Generic renderer.
pub struct Renderer {
    device: types::Device,
    encoders: Vec<Encoder>,
    factory: Factory,
    main_target: Target,
}

impl Renderer {
    /// Creates a new renderer with the given device and factory.
    pub fn new(dev: types::Device, mut fac: Factory, main: Target) -> Self {
        let num_cores = num_cpus::get();

        let encoders = (0..num_cores)
            .map(|_| fac.create_command_buffer().into())
            .collect();

        Renderer {
            device: dev,
            encoders: encoders,
            factory: fac,
            main_target: main,
        }
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh<'a, V>(&'a mut self, verts: &'a [V]) -> MeshBuilder<'a, V>
        where V: VertexFormat + 'a
    {
        MeshBuilder::new(&mut self.factory, &verts)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipeline(&mut self) -> PipelineBuilder {
        PipelineBuilder::new(&mut self.factory, self.main_target.clone())
    }

    /// Draws a scene with the given pipeline.
    pub fn draw(&mut self, pipe: &Pipeline, delta: Duration) {
        use gfx::Device;
        use rayon::prelude::*;

        let dt = (delta.subsec_nanos() as f64 * 1e-9) + (delta.as_secs() as f64);

        pipe.stages()
            .par_iter()
            .zip(self.encoders.as_mut_slice())
            .for_each(|(stage, ref mut enc)| stage.apply(enc, dt));

        for enc in self.encoders.as_mut_slice() {
            enc.flush(&mut self.device);
        }

        self.device.cleanup();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use gfx::Device;
        self.device.cleanup();
    }
}

impl From<(types::Device, Factory, Target)> for Renderer {
    fn from((dev, fac, main): (types::Device, Factory, Target)) -> Renderer {
        Renderer::new(dev, fac, main)
    }
}

/// Builds a new renderer.
pub struct RendererBuilder {
    window: winit::WindowBuilder,
}

impl RendererBuilder {
    /// Constructs a new RendererBuilder from the given WindowBuilder.
    pub fn new(wb: winit::WindowBuilder) -> Self {
        RendererBuilder { window: wb }
    }

    /// Builds a new renderer.
    #[cfg(feature = "opengl")]
    pub fn build(self) -> Result<(Window, Renderer)> {
        use glutin::{GlProfile, GlRequest, WindowBuilder};
        let wb = WindowBuilder::from_winit_builder(self.window)
            .with_gl_profile(GlProfile::Core)
            .with_gl(GlRequest::Latest);

        let (win, dev, fac, color, depth) = gfx_window_glutin::init::<ColorFormat, DepthFormat>(wb);

        let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
        let main_target: Target = (vec![color], depth, size).into();

        Ok((win, (dev, fac, main_target).into()))
    }

    /// Builds a new renderer.
    #[cfg(all(feature = "d3d11", target_os = "windows"))]
    pub fn build(self) -> Result<(Window, Renderer)> {
        let (win, dev, mut fac, color) = gfx_window_dxgi::init::<ColorFormat>(self.window).unwrap();
        let dev = gfx_device_dx11::Deferred::from(dev);

        let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
        let (w, h) = (size.0 as u16, size.1 as u16);
        let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
        let main_target: Target = (vec![color], depth, size).into();

        Ok((win, (dev, fac, main_target).into()))
    }
}
