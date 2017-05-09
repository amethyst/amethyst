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
//! let sphere = renderer.create_mesh(Mesh::new(&verts)
//!     .with_ambient_texture(Rgba(1.0, 0.5, 0.2, 1.0))
//!     .with_diffuse_texture(Rgba(0.7, 0.3, 0.1, 1.0)))
//!     .unwrap();
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
//! }
//! ```

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

pub use cam::{Camera, Projection};
pub use color::Rgba;
pub use error::{Error, Result};
pub use light::Light;
pub use mesh::{Mesh, MeshBuilder};
pub use mtl::{Material, MaterialBuilder};
pub use pipe::{Pipeline, PipelineBuilder, Stage, Target};
pub use scene::{LightIter, MeshIter, Scene};
pub use tex::{Texture, TextureBuilder};
pub use vertex::VertexFormat;

use pipe::{ColorBuffer, DepthBuffer};
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

static DRAW_VERTEX_SRC: &[u8] = b"
    #version 150 core
    layout (std140) uniform cb_VertexArgs {
        uniform mat4 u_Proj;
        uniform mat4 u_View;
        uniform mat4 u_Model;
    };
    in vec3 a_Normal;
    in vec3 a_Pos;
    in vec2 a_TexCoord;
    out vec3 v_Normal;
    out vec2 v_TexCoord;
    void main() {
        v_TexCoord = a_TexCoord;
        v_Normal = mat3(u_Model) * a_Normal;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
";

static DRAW_FRAGMENT_SRC: &[u8] = b"
    #version 150 core
    uniform sampler2D t_Ka;
    uniform sampler2D t_Kd;
    uniform sampler2D t_Ks;
    in vec3 v_Normal;
    in vec2 v_TexCoord;
    out vec4 o_Ka;
    out vec4 o_Kd;
    out vec4 o_Ks;
    out vec4 o_Normal;
    void main() {
        o_Ka = texture(t_Ka, v_TexCoord);
        o_Kd = texture(t_Kd, v_TexCoord);
        o_Ks = texture(t_Ks, v_TexCoord);
        o_Normal = vec4(normalize(v_Normal), 0.);
    }
";

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
    pub fn new(el: &EventsLoop) -> Result<Renderer> {
        let wb = WindowBuilder::new().with_title("Amethyst");
        Renderer::from_winit_builder(wb, el)
    }

    /// Creates a new renderer with the given `winit::WindowBuilder`.
    pub fn from_winit_builder(wb: WindowBuilder, el: &EventsLoop) -> Result<Renderer> {
        let Backend(dev, mut fac, main, win) = init_backend(wb, el)?;

        let num_cores = num_cpus::get();
        let encoders = (0..num_cores)
            .map(|_| fac.create_command_buffer().into())
            .collect();

        let cfg = rayon::Configuration::new().num_threads(num_cores);
        let pool = rayon::ThreadPool::new(cfg)
            .map_err(|e| Error::PoolCreation(e))?;

        let init = pipe::EffectBuilder::new()
            .with_simple_prog(DRAW_VERTEX_SRC, DRAW_FRAGMENT_SRC)
            .build(&mut fac, &main).expect("Blah");

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
        use rayon::prelude::*;

        {
            let encoders = self.encoders.as_mut_slice();
            self.pool.install(|| {
                pipe.stages()
                    .par_iter()
                    .zip(encoders)
                    .filter(|&(stage, _)| stage.is_enabled())
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
