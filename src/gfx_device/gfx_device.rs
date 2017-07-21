//! Very light wrapper around GFX.

use ecs::{Join, World, resources};
use engine::WindowEvent;
use gfx::Device;
use gfx_device::gfx_types;
use gfx_device::gfx_types::{CommandBuffer, Resources, Window};
use renderer::{Fragment, Pipeline, Renderer, Scene};

/// Holds all graphics resources required to render a `Scene`/`Pipeline` pair,
/// except `MainTarget`.
pub struct GfxDevice {
    /// Handles drawing output to the screen.
    pub device: gfx_types::Device,
    /// Processes and renders scenes.
    pub renderer: Renderer<Resources, CommandBuffer>,
    /// An application window.
    pub window: Window,
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
    pub fn render_world(&mut self, world: &mut World, pipe: &Pipeline) {
        use ecs::components::{Renderable, Transform};
        use ecs::resources::Projection;
        use renderer::{AmbientLight, Camera, DirectionalLight, PointLight};

        let camera = world.read_resource::<resources::Camera>();
        let proj_mat = match camera.proj {
            Projection::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => Camera::perspective(fov, aspect_ratio, near, far),
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => Camera::orthographic(left, right, bottom, top, near, far),
        };

        let eye = camera.eye;
        let target = camera.target;
        let up = camera.up;
        let view_mat = Camera::look_at(eye, target, up);
        let camera = Camera::new(proj_mat, view_mat);
        let mut scene = Scene::<Resources>::new(camera);

        let entities = world.entities();
        let renderables = world.read::<Renderable>();
        let global_transforms = world.read::<Transform>();

        // Add all entities with `Renderable` components attached to them to
        // the scene.
        for (rend, entity) in (&renderables, &*entities).join() {
            let global_trans = match global_transforms.get(entity) {
                Some(gt) => *gt,
                None => Transform::default(),
            };

            if let Some(frag) = unwrap_renderable(rend, &global_trans) {
                scene.fragments.push(frag);
            }
        }

        // Add all lights to the scene.
        scene.point_lights.extend(world.read::<PointLight>().join());
        scene
            .directional_lights
            .extend(world.read::<DirectionalLight>().join());

        let ambient_light = world.read_resource::<AmbientLight>();
        scene.ambient_light = ambient_light.power;

        // Render the final scene.
        self.renderer.submit(pipe, &scene, &mut self.device);
        self.window.swap_buffers().unwrap();
        self.device.cleanup();

        // Function that creates `Fragment`s from `Renderable`, `Transform` pairs.
        fn unwrap_renderable(rend: &Renderable,
                             global_trans: &Transform)
                             -> Option<Fragment<Resources>> {
            let mesh = &rend.mesh;
            Some(Fragment {
                     transform: global_trans.clone().into(),
                     buffer: mesh.buffer.clone(),
                     slice: mesh.slice.clone(),
                     ka: (&rend.ambient).clone(),
                     kd: (&rend.diffuse).clone(),
                     ks: (&rend.specular).clone(),
                     ns: rend.specular_exponent,
                 })
        }
    }

    /// Poll events from `GfxDevice`.
    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        if cfg!(feature = "opengl") {
            self.window.poll_events().map(WindowEvent::new).collect()
        } else {
            unimplemented!()
        }
    }
}
