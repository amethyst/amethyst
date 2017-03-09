//! A highly parallel rendering engine developed for the Amethyst game engine.
//!
//! # Example
//!
//! ```ignore
//! let wb = winit::WindowBuilder::new()
//!     .with_title("Amethyst Renderer Demo")
//!     .with_dimensions(800, 600);
//!
//! let mut renderer = Renderer::from_winit_builder(wb)
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
//!     .add_mesh(sphere)
//!     .add_light(light);
//!
//! let pipe = renderer.create_pipeline()
//!     .with_target(Target::new("gbuffer")
//!         .with_num_color_bufs(4)
//!         .with_depth_buf(true))
//!     .with_stage(Stage::with_target("gbuffer")
//!         .with_pass(ClearTarget::with_values(Rgba::default(), 0.0))
//!         .with_pass(DrawFlat::with_camera("main_camera")))
//!     .with_stage(Stage::with_target("main")
//!         .with_pass(BlitBuffer::color_buf("gbuffer", 2))
//!         .with_pass(DeferredLighting::new("main_camera", "gbuffer", "scene")))
//!     .build()
//!     .expect("Could not build pipeline");
//!
//! 'main: loop {
//!     for event in renderer.window().poll_events() {
//!         match event {
//!             winit::Event::Closed => break 'main,
//!             _ => (),
//!         }
//!     }
//!
//!     renderer.draw(&scene, &pipe, delta);
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
pub use light::Light;
pub use mesh::{Mesh, MeshBuilder};
pub use pass::Pass;
pub use pipe::{Pipeline, PipelineBuilder, Target, TargetBuilder, Stage, StageBuilder};
pub use scene::Scene;
pub use types::VertexFormat;

use std::time::Duration;
use types::{Buffer, ColorFormat, DepthFormat, Encoder, Factory, Resources, Slice, Window};

pub mod color;
pub mod light;
pub mod pass;
pub mod vertex;

mod error;
mod mesh;
mod pipe;
mod scene;
mod types;

/// Generic renderer.
pub struct Renderer {
    device: types::Device,
    encoders: Vec<Encoder>,
    factory: Factory,
    main_target: Target,
    window: Window,
}

impl Renderer {
    /// Creates a new renderer with the given device and factory.
    pub fn from_winit_builder(builder: winit::WindowBuilder) -> Result<Renderer> {
        let Backend(dev, mut fac, main, win) = init_backend(builder)?;

        let num_cores = num_cpus::get();
        let encoders = (0..num_cores)
            .map(|_| fac.create_command_buffer().into())
            .collect();

        Ok(Renderer {
            device: dev,
            encoders: encoders,
            factory: fac,
            main_target: main,
            window: win,
        })
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh<'a, V>(&'a mut self, verts: &'a [V]) -> MeshBuilder<'a, V>
        where V: VertexFormat + 'a
    {
        MeshBuilder::new(&mut self.factory, verts)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipeline(&mut self) -> PipelineBuilder {
        PipelineBuilder::new(&mut self.factory, self.main_target.clone())
    }

    /// Returns an immutable reference to the renderer window.
    pub fn window(&self) -> &winit::Window {
        self.window.as_winit_window()
    }

    /// Draws a scene with the given pipeline.
    pub fn draw(&mut self, scene: &Scene, pipe: &Pipeline, delta: Duration) {
        use gfx::Device;
        use rayon::prelude::*;

        let dt = (delta.subsec_nanos() as f64 * 1e-9) + (delta.as_secs() as f64);

        pipe.stages()
            .par_iter()
            .zip(self.encoders.as_mut_slice())
            .for_each(|(stage, ref mut enc)| stage.apply(enc, scene, dt));

        for enc in self.encoders.as_mut_slice() {
            enc.flush(&mut self.device);
        }

        self.device.cleanup();

        #[cfg(feature = "opengl")]
        self.window.swap_buffers().expect("OpenGL context has been lost");
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use gfx::Device;
        self.device.cleanup();
    }
}

/// Represents a graphics backend for the renderer.
struct Backend(pub types::Device, pub Factory, pub Target, pub Window);

/// Creates the OpenGL backend.
#[cfg(feature = "opengl")]
fn init_backend(builder: winit::WindowBuilder) -> Result<Backend> {
    use glutin::{GlProfile, GlRequest, WindowBuilder};
    use gfx_window_glutin as win;

    let wb = WindowBuilder::from_winit_builder(builder)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Latest);

    let (win, dev, fac, color, depth) = win::init::<ColorFormat, DepthFormat>(wb);
    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let main_target: Target = (vec![color], depth, size).into();

    Ok(Backend(dev, fac, main_target, win))
}

/// Creates the Direct3D 11 backend.
#[cfg(all(feature = "d3d11", target_os = "windows"))]
fn init_backend(builder: winit::WindowBuilder) -> Result<Backend> {
    use gfx_window_dxgi as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(builder).unwrap();
    let dev = gfx_device_dx11::Deferred::from(dev);

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target: Target = (vec![color], depth, size).into();

    Ok(Backend(dev, fac, main_target, win))
}
