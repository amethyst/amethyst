extern crate specs;

use renderer;
use self::specs::Join;
use event::WindowEvent;
use gfx_device::gfx::Device;
use gfx_device::gfx_types;

/// This struct holds all the graphics resources (except `MainTarget`) required to render a `Scene`, `Pipeline` pair.
pub struct GfxDevice {
    pub window: gfx_types::Window,
    pub device: gfx_types::Device,
    pub renderer: renderer::Renderer<gfx_types::Resources, gfx_types::CommandBuffer>,
}

impl GfxDevice {
    /// Get screen dimensions.
    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        #[cfg(feature="opengl")]
        return self.window.get_inner_size();
        #[cfg(all(windows, feature="direct3d"))]
        unimplemented!();
    }

    /// Render all `Entity`s with `Renderable` components in `World`.
    pub fn render_world(&mut self, world: &mut self::specs::World, pipeline: &renderer::Pipeline) {
        use ecs::components::transform::Transform;
        use ecs::components::rendering::Renderable;
        use ecs::resources::camera::{Projection, Camera};
        use renderer::Fragment;
        let camera = world.read_resource::<Camera>().clone();

        let projection_mat = match camera.projection {
            Projection::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            } => renderer::Camera::perspective(fov, aspect_ratio, near, far),
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => renderer::Camera::orthographic(left, right, bottom, top, near, far),
        };
        let eye = camera.eye;
        let target = camera.target;
        let up = camera.up;
        let view_mat = renderer::Camera::look_at(eye, target, up);
        let camera = renderer::Camera::new(projection_mat, view_mat);

        let mut scene = renderer::Scene::<gfx_types::Resources>::new(camera);
        let entities = world.entities();
        let renderables = world.read::<Renderable>();
        let global_transforms = world.read::<Transform>();
        // Add all `Entity`s with `Renderable` components attached to them to the `Scene`.
        for (renderable, entity) in (&renderables, &entities).iter() {
            let global_transform = match global_transforms.get(entity) {
                Some(global_transform) => global_transform.clone(),
                None => Transform::default(),
            };
            if let Some(fragment) = unwrap_renderable(renderable, &global_transform) {
                scene.fragments.push(fragment);
            }
        }

        // Add all `Light`s to the `Scene`.
        scene.point_lights.extend(world.read::<renderer::PointLight>().iter());
        scene.directional_lights.extend(world.read::<renderer::DirectionalLight>().iter());

        let ambient_light = world.read_resource::<renderer::AmbientLight>();
        scene.ambient_light = ambient_light.power;

        // Render the `Scene`.
        self.renderer.submit(pipeline, &scene, &mut self.device);
        self.window.swap_buffers().unwrap();
        self.device.cleanup();
        // Function that creates `Fragment`s from `Renderable`, `Transform` pairs.
        fn unwrap_renderable(renderable: &Renderable, global_transform: &Transform) -> Option<Fragment<gfx_types::Resources>> {
            let mesh = &renderable.mesh;
            Some(Fragment {
                transform: global_transform.clone().into(),
                buffer: mesh.buffer.clone(),
                slice: mesh.slice.clone(),
                ka: (&renderable.ambient).clone(),
                kd: (&renderable.diffuse).clone(),
                ks: (&renderable.specular).clone(),
                ns: renderable.specular_exponent,
            })
        }
    }

    /// Poll events from `GfxDevice`.
    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        #[cfg(feature="opengl")]
        {
            let mut events = vec![];
            for event in self.window.poll_events() {
                let event = WindowEvent::new(event);
                events.push(event);
            }
            events
        }
        #[cfg(all(windows, feature="direct3d"))]
        unimplemented!();
    }
}
