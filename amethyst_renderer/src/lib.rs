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
//! ```ignore
//! let event_loop = winit::EventsLoop::new();
//! let mut renderer = Renderer::new(&event_loop).unwrap();
//! let pipe = renderer.create_pipe(Pipeline::deferred()).unwrap();
//!
//! let verts = some_sphere_gen_func();
//! let sphere = renderer.create_mesh(Mesh::build(&verts)).unwrap();
//!
//! let light = PointLight::default();
//!
//! let scene = Scene::default()
//!     .add_mesh("ball", sphere)
//!     .add_light("lamp", light);
//!
//! event_loop.run_forever(|e| {
//!     let winit::Event::WindowEvent { event, .. } = e;
//!     match event {
//!         winit::WindowEvent::Closed => event_loop.interrupt(),
//!         _ => (),
//!     }
//!
//!     renderer.draw(&scene, &pipe, dt);
//! };
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
pub use error::{Error, Result};
pub use light::Light;
pub use mesh::{Mesh, MeshBuilder};
pub use mtl::{Material, MaterialBuilder};
pub use pipe::{Pipeline, PipelineBuilder, Stage, Target};
pub use scene::{Model, Scene};
pub use tex::{Texture, TextureBuilder};
pub use vertex::VertexFormat;

use pipe::{ColorBuffer, DepthBuffer};
use pipe::pass::Pass;
use std::sync::Arc;
use std::time::Duration;
use types::{ColorFormat, DepthFormat, Encoder, Factory, Window};
use winit::WindowBuilder;

#[cfg(feature = "opengl")]
use glutin::EventsLoop;
#[cfg(not(feature = "opengl"))]
use winit::EventsLoop;

pub mod light;
pub mod pass;
pub mod prelude;
pub mod pipe;
pub mod vertex;

mod cam;
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
    pool: Arc<rayon::ThreadPool>,
    window: Window,
}

impl Renderer {
    /// Creates a new renderer with default window settings.
    pub fn new(el: &EventsLoop) -> Result<Renderer> {
        Renderer::build(el).finish()
    }

    #[allow(missing_docs)]
    pub fn build(el: &EventsLoop) -> RendererBuilder {
        RendererBuilder::new(el)
    }

    /// Builds a new mesh from the given vertices.
    pub fn create_mesh(&mut self, mb: MeshBuilder) -> Result<Mesh> {
        mb.finish(&mut self.factory)
    }

    /// Builds a new renderer pipeline.
    pub fn create_pipe(&mut self, pb: PipelineBuilder) -> Result<Pipeline> {
        pb.finish(&mut self.factory, &self.main_target)
    }

    /// Builds a new texture resource.
    pub fn create_texture(&mut self, tb: TextureBuilder) -> Result<Texture> {
        tb.finish(&mut self.factory)
    }

    /// Builds a new material resource.
    pub fn create_material(&mut self, mb: MaterialBuilder) -> Result<Material> {
        mb.finish(&mut self.factory)
    }

    /// Draws a scene with the given pipeline.
    pub fn draw(&mut self, scene: &Scene, pipe: &Pipeline, _delta: Duration) {
        use gfx::Device;
        use rayon::prelude::*;

        let Renderer {
            ref mut device,
            ref mut encoders,
            ref mut factory,
            ref pool,
            ref mut window,
            ..
        } = *self;

        let num_threads = pool.current_num_threads();
        let encoders_required = pipe.stages()
            .iter()
            .filter(|stage| stage.is_enabled())
            .map(|stage| stage.encoders_required(num_threads))
            .sum::<usize>();

        let encoders_count = encoders.len();
        if encoders_count < encoders_required {
            encoders.extend((encoders_count..encoders_required).map(|_| factory.create_command_buffer().into()))
        }

        {
            let mut encoders = encoders.as_mut_slice();
            let mut model = Vec::new();
            let mut light = Vec::new();
            for stage in pipe.stages().iter().filter(|stage| stage.is_enabled()) {
                for pass in stage.passes().iter() {
                    match *pass {
                        Pass::Basic(ref pass) => {
                            let enc = {
                                let slice = encoders;
                                let (first, left) = slice.split_first_mut().unwrap();
                                encoders = left;
                                first
                            };
                            pass.apply(enc, &stage.target());
                        }
                        Pass::Simple(ref pass) => {
                            let enc = {
                                let slice = encoders;
                                let (first, left) = slice.split_first_mut().unwrap();
                                encoders = left;
                                first
                            };
                            pass.apply(enc, &stage.target(), scene)
                        }
                        Pass::Model(ref pass) => {
                            let mut fragments_iter = scene.par_chunks_models(num_threads);
                            let enc = {
                                let slice = encoders;
                                let (count, left) = slice.split_at_mut(fragments_iter.len());
                                encoders = left;
                                count
                            };
                            model.push(fragments_iter
                                           .zip(enc)
                                           .map(move |(models, enc)| for model in models {
                                                    pass.apply(enc, &stage.target(), scene, model);
                                                }));
                        }
                        Pass::Light(ref pass) => {
                            let mut fragments_iter = scene.par_chunks_lights(num_threads);
                            let enc = {
                                let slice = encoders;
                                let (count, left) = slice.split_at_mut(fragments_iter.len());
                                encoders = left;
                                count
                            };
                            light.push(fragments_iter
                                           .zip(enc)
                                           .map(move |(lights, enc)| for light in lights {
                                                    pass.apply(enc, &stage.target(), scene, light);
                                                }));
                        }
                    }
                }
            }
            pool.install(move || {
                model.into_par_iter()
                    .flat_map(|i|i)
                    .chain(light.into_par_iter().flat_map(|i|i))
                    .for_each(|()| {});
            });
        }

        for enc in encoders.iter_mut() {
            enc.flush(device);
        }
        device.cleanup();
        #[cfg(feature = "opengl")]
        window.swap_buffers().expect("OpenGL context has been lost");
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

#[allow(missing_docs)]
pub struct RendererBuilder<'a> {
    events: &'a EventsLoop,
    pool: Option<Arc<rayon::ThreadPool>>,
    winit_builder: WindowBuilder,
}

impl<'a> RendererBuilder<'a> {
    #[allow(missing_docs)]
    pub fn new(el: &'a EventsLoop) -> RendererBuilder<'a> {
        RendererBuilder {
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
    pub fn with_pool(mut self, pool: Arc<rayon::ThreadPool>) -> Self {
        self.pool = Some(pool);
        self
    }

    #[allow(missing_docs)]
    pub fn finish(self) -> Result<Renderer> {
        let Backend(dev, mut fac, main, win) = init_backend(self.winit_builder, self.events)?;

        let num_cores = num_cpus::get();
        let pool = self.pool
            .map(|p| Ok(p))
            .unwrap_or_else(|| {
                                let cfg = rayon::Configuration::new().num_threads(num_cores);
                                rayon::ThreadPool::new(cfg)
                                    .map(|p| Arc::new(p))
                                    .map_err(|e| Error::PoolCreation(e))
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
fn init_backend(wb: WindowBuilder, el: &EventsLoop) -> Result<Backend> {
    use gfx_window_dxgi as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(wb, el).unwrap();
    let dev = gfx_device_dx11::Deferred::from(dev);

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target = Target::from((
        vec![ColorBuffer {
                                             as_input: None,
                                             as_output: color,
                                         }],
                                    DepthBuffer {
                                        as_input: None,
                                        as_output: depth,
                                    },
                                    size));

    Ok(Backend(dev, fac, main_target, win))
}

#[cfg(all(feature = "metal", target_os = "macos"))]
fn init_backend(wb: WindowBuilder, el: &EventsLoop) -> Result<Backend> {
    use gfx_window_metal as win;

    let (win, dev, mut fac, color) = win::init::<ColorFormat>(wb, el).unwrap();

    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let (w, h) = (size.0 as u16, size.1 as u16);
    let depth = fac.create_depth_stencil_view_only::<DepthFormat>(w, h)?;
    let main_target = Target::from((
        vec![ColorBuffer {
            as_input: None,
            as_output: color,
        }],
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size));

    Ok(Backend(dev, fac, main_target, win))
}

/// Creates the OpenGL backend.
#[cfg(feature = "opengl")]
fn init_backend(wb: WindowBuilder, el: &EventsLoop) -> Result<Backend> {
    use glutin::{GlProfile, GlRequest};
    use gfx_window_glutin as win;

    let wb = glutin::WindowBuilder::from_winit_builder(wb)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Latest);

    let (win, dev, fac, color, depth) = win::init::<ColorFormat, DepthFormat>(wb, el);
    let size = win.get_inner_size_points().ok_or(Error::WindowDestroyed)?;
    let main_target = Target::from((
        vec![ColorBuffer {
            as_input: None,
            as_output: color,
        }],
        DepthBuffer {
            as_input: None,
            as_output: depth,
        },
        size));

    Ok(Backend(dev, fac, main_target, win))
}
