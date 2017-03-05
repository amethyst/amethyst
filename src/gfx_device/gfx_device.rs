//! Very light wrapper around GFX.
extern crate gfx;
extern crate specs;
extern crate scoped_threadpool;

use ecs::{Join, World};
use engine::WindowEvent;
use gfx::Device;
use gfx_device::gfx_types;
use gfx_device::gfx_types::{CommandBuffer, Resources, Window};
use renderer::{Renderer, AmbientLight};
use renderer;

/// Holds all graphics resources required to render a slice of `Fragment`s and a `Scene`/`Pipeline` pair,
pub struct GfxDevice {
    /// Handles drawing output to the screen.
    pub device: gfx_types::Device,
    /// Processes and renders scenes.
    pub renderer: Renderer<Resources, CommandBuffer>,
    /// An application window.
    pub window: Window,
    /// Main color target.
    pub main_color: gfx::handle::RenderTargetView<gfx_types::Resources, renderer::target::ColorFormat>,
    /// Main depth target.
    pub main_depth: gfx::handle::DepthStencilView<gfx_types::Resources, renderer::target::DepthFormat>,
    /// `Encoder`s vector used for parallel rendering.
    pub encoders: Vec<gfx_types::Encoder>,
}

impl GfxDevice {
    /// Returns the window's dimensions in pixels.
    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        if cfg!(feature = "opengl") {
            self.window.get_inner_size()
        } else {
            unimplemented!()
        }
    }

    /// Render all `Entity`s with `Renderable` components in `World`.
    pub fn render_world(&mut self,
                        world: &mut World,
                        pipe: &renderer::Pipeline,
                        pool: &mut self::scoped_threadpool::Pool) {
        use std::ops::Index;
        use ecs::components::Transform;
        use ecs::components::Renderable;
        use ecs::resources::{Projection, Camera};
        use ecs::resources::ClearColor;
        use renderer::Fragment;

        let camera = world.read_resource::<Camera>().clone();
        let proj_mat = match camera.proj {
            Projection::Perspective { fov, aspect_ratio, near, far } => {
                renderer::Camera::perspective(fov, aspect_ratio, near, far)
            }
            Projection::Orthographic { left, right, bottom, top, near, far } => {
                renderer::Camera::orthographic(left, right, bottom, top, near, far)
            }
        };

        let eye = camera.eye;
        let target = camera.target;
        let up = camera.up;
        let view_mat = renderer::Camera::look_at(eye, target, up);
        let camera = renderer::Camera::new(proj_mat, view_mat);

        let mut scene = renderer::Scene::new(camera);
        let entities = world.entities();
        let renderables = world.read::<Renderable>();
        let global_transforms = world.read::<Transform>();

        // Get `Fragment`s from `Renderable`s.
        let mut fragments = vec![];
        for (renderable, entity) in (&renderables, &entities).iter() {
            let global_transform = match global_transforms.get(entity) {
                Some(global_transform) => global_transform.clone(),
                None => Transform::default(),
            };
            let fragment = unwrap_renderable(renderable, &global_transform);
            fragments.push(fragment);
        }

        // Pack `Fragment`s into num_encoders equal chunks (+ remainder in first chunk).
        let num_encoders = self.encoders.len();
        let num_fragments = fragments.len();
        let chunk_size = num_fragments / num_encoders;
        let remainder_size = num_fragments % num_encoders;
        // A vector of fragments chunks. A reference to a chunk is passed to each thread in the threadpool.
        let mut fragments_chunks: Vec<Vec<Fragment<gfx_types::Resources>>> = vec![];
        // 0th chunk contains chunk_size + remainder_size `Fragment`s.
        // Preallocate memory for 0th fragment chunk.
        let fragments_chunk0 = Vec::<Fragment<gfx_types::Resources>>::with_capacity(chunk_size + remainder_size);
        fragments_chunks.push(fragments_chunk0);
        // Preallocate memory for fragment chunks going from 1 to num_encoders.
        for _ in 1..num_encoders {
            let fragments_chunk = Vec::<Fragment<gfx_types::Resources>>::with_capacity(chunk_size);
            fragments_chunks.push(fragments_chunk);
        }
        // Populate 0th fragments_chunk.
        let fragments_chunk0 = fragments.index(0..chunk_size + remainder_size);
        fragments_chunks[0].extend_from_slice(fragments_chunk0);
        // Populate all other fragments_chunks.
        for i in 1..fragments_chunks.len() {
            let chunk_begin = chunk_size + remainder_size + (i-1) * chunk_size;
            let chunk_end = chunk_size + remainder_size + i * chunk_size;
            let fragments_chunk = fragments.index(chunk_begin..chunk_end);
            fragments_chunks[i].extend_from_slice(fragments_chunk);
        }
        // Add all `Light`s to the `Scene`.
        scene.point_lights.extend(world.read::<renderer::PointLight>().iter());
        scene.directional_lights.extend(world.read::<renderer::DirectionalLight>().iter());

        let ambient_light = world.read_resource::<AmbientLight>();
        scene.ambient_light = ambient_light.power;

        let clear_color = world.read_resource::<ClearColor>();
        // Clear screen. At the end self.encoder[0] will be flushed first because it has the lowest index.
        self.encoders[0].clear(&self.main_color, clear_color.clear_color);
        self.encoders[0].clear_depth(&self.main_depth, clear_color.clear_depth);
        // Render the `Scene` (fill up `Encoder`s) in parallel using a `scoped_threadpool::Pool`.
        pool.scoped(|scope| {
            let renderer = &self.renderer;
            let pipe = pipe;
            let scene = &scene;
            let encoders = &mut self.encoders;
            let fragments_chunks = &fragments_chunks;
            for (fragments_chunk, encoder) in fragments_chunks.into_iter().zip(encoders.into_iter()) {
                scope.execute(move || {
                    renderer.submit(encoder, pipe, fragments_chunk.as_slice(), scene);
                });
            }
        });
        // Flush all `Encoder`s.
        for encoder in &mut self.encoders {
            encoder.flush(&mut self.device);
        }
        self.window.swap_buffers().unwrap();
        self.device.cleanup();

        // Function that creates `Fragment`s from `Renderable`, `Transform` pairs.
        fn unwrap_renderable(renderable: &Renderable,
                             global_transform: &Transform)
                             -> Fragment<gfx_types::Resources> {
            let mesh = &renderable.mesh;
            Fragment {
                transform: global_transform.clone().into(),
                buffer: mesh.buffer.clone(),
                slice: mesh.slice.clone(),
                ka: (&renderable.ambient).clone(),
                kd: (&renderable.diffuse).clone(),
                ks: (&renderable.specular).clone(),
                ns: renderable.specular_exponent,
            }
        }
    }

    /// Get a `ColorBuffer` containing main_color and main_depth targets.
    pub fn get_main_target(&self) -> renderer::target::ColorBuffer<gfx_types::Resources> {
        renderer::target::ColorBuffer {
            color: self.main_color.clone(),
            output_depth: self.main_depth.clone(),
        }
    }

    /// Poll events from `GfxDevice`.
    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        if cfg!(feature = "opengl") {
            self.window.poll_events().map(|e| WindowEvent::new(e)).collect()
        } else {
            unimplemented!()
        }
    }
}
