//! A highly parallel rendering engine developed for the Amethyst game engine.
//!
//! # Example
//!
//! ```ignore
//! let mut renderer = Renderer::new().unwrap();
//!
//! let verts = some_sphere_gen_func();
//! let sphere = renderer.create_mesh(&verts)
//!     .with_ambient_texture(Rgba(1.0, 0.5, 0.2, 1.0))
//!     .with_diffuse_texture(Rgba(0.7, 0.3, 0.1, 1.0))
//!     .build()
//!     .unwrap()
//!
//! let light = PointLight::default();
//!
//! let scene = Scene::new()
//!     .add_mesh("ball", sphere)
//!     .add_light("lamp", light);
//!
//! let pipe = pipe::deferred(&mut renderer).unwrap();
//!
//! 'main: loop {
//!     for event in renderer.window().poll_events() {
//!         match event {
//!             winit::Event::Closed => break 'main,
//!             _ => (),
//!         }
//!     }
//!
//!     renderer.draw(&scene, &pipe, dt);
//! }
//! ```

#![crate_name = "amethyst_renderer"]
#![crate_type = "lib"]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://tinyurl.com/jtmm43a")]

extern crate cgmath;
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

pub use color::Rgba;
pub use error::{Error, Result};
pub use light::Light;
pub use mesh::{Mesh, MeshBuilder};
pub use mtl::Material;
pub use pipe::{Pipeline, PipelineBuilder, Target, Stage};
pub use scene::Scene;
pub use tex::{Texture, TextureBuilder};
pub use vertex::VertexFormat;

use pipe::{ColorBuffer, DepthBuffer};
use std::sync::Arc;
use std::time::Duration;
use types::{ColorFormat, DepthFormat, Encoder, Factory, Window};

pub mod light;
pub mod pass;
pub mod prelude;
pub mod pipe;
pub mod vertex;

mod color;
mod error;
mod mesh;
mod mtl;
mod scene;
mod tex;
mod types;

/// Generic renderer.
pub struct Renderer {
    device: types::Device,
    encoders: Vec<Encoder>,
    factory: Factory,
    main_target: Arc<Target>,
    pool: rayon::ThreadPool,
    window: Window,
}

impl Renderer {
    /// Creates a new renderer with default window settings.
    pub fn new() -> Result<Renderer> {
        let wb = winit::WindowBuilder::new();
        Renderer::from_winit_builder(wb)
    }

    /// Creates a new renderer with the given `winit::WindowBuilder`.
    pub fn from_winit_builder(builder: winit::WindowBuilder) -> Result<Renderer> {
        let Backend(dev, mut fac, main, win) = init_backend(builder)?;

        let num_cores = num_cpus::get();
        let encoders = (0..num_cores)
            .map(|_| fac.create_command_buffer().into())
            .collect();

        let cfg = rayon::Configuration::new().num_threads(num_cores);
        let pool = rayon::ThreadPool::new(cfg).map_err(|e| Error::PoolCreation(e))?;

        Ok(Renderer {
            device: dev,
            encoders: encoders,
            factory: fac,
            main_target: Arc::new(main),
            pool: pool,
            window: win,
        })
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh<V>(&mut self, verts: &'static [V]) -> MeshBuilder
        where V: VertexFormat
    {
        MeshBuilder::new(&mut self.factory, verts)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipeline(&mut self) -> PipelineBuilder {
        PipelineBuilder::new(&mut self.factory, self.main_target.clone())
    }

    /// Builds a new texture resource.
    pub fn create_texture(&mut self) -> TextureBuilder {
        TextureBuilder::new(&mut self.factory)
    }

    /// Draws a scene with the given pipeline.
    pub fn draw(&mut self, scene: &Scene, pipe: &Pipeline, _delta: Duration) {
        use gfx::Device;
        use rayon::prelude::*;

        {
            let encoders = self.encoders.as_mut_slice();
            self.pool.install(|| {
                    pipe.stages()
                        .par_iter()
                        .zip(encoders)
                        .for_each(|(stage, enc)| stage.apply(enc, scene));
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
        self.window.as_winit_window()
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

/// Creates the Direct3D 11 backend.
#[cfg(all(feature = "d3d11", target_os = "windows"))]
fn init_backend(builder: winit::WindowBuilder) -> Result<Backend> {
    use gfx_window_dxgi as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(builder).unwrap();
    let dev = gfx_device_dx11::Deferred::from(dev);

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target = Target::from((vec![
        ColorBuffer {
            as_input: None,
            as_output: color,
        }],
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size,
    ));

    Ok(Backend(dev, fac, main_target, win))
}

#[cfg(all(feature = "metal", target_os = "macos"))]
fn init_backend(builder: winit::WindowBuilder) -> Result<Backend> {
    use gfx_window_metal as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(builder).unwrap();

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target = Target::from((vec![
        ColorBuffer {
            as_input: None,
            as_output: color,
        }],
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size,
    ));

    Ok(Backend(dev, fac, main_target, win))
}

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
    let main_target = Target::from((vec![
        ColorBuffer {
            as_input: None,
            as_output: color,
        }],
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size,
    ));

    Ok(Backend(dev, fac, main_target, win))
}
