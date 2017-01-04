macro_rules! unwind_gfx_device_inner_mut {
    ($variable:expr, $field1:ident, $expr_field:expr, $expr_null:expr) => {
        match $variable {
            GfxDeviceInner::OpenGL {
                ref mut $field1,
                ..
            } => $expr_field,
            #[cfg(windows)]
            GfxDeviceInner::Direct3D { } => unimplemented!(),
            GfxDeviceInner::Null => $expr_null,
        }
    };
}

extern crate gfx_device_gl;

use renderer;
use ecs;
use ecs::Join;
use event::WindowEvent;
use world_resources;
use gfx_device::gfx_device_inner::GfxDeviceInner;
use gfx_device::gfx::Device;

/// This struct holds all the graphics resources (except `MainTarget`) required to render a `Scene`, `Pipeline` pair.
pub struct GfxDevice {
    gfx_device_inner: GfxDeviceInner,
}

impl GfxDevice {
    /// Create a new `GfxDevice` from `GfxDeviceInner`.
    pub fn new(gfx_device_inner: GfxDeviceInner) -> GfxDevice {
        GfxDevice { gfx_device_inner: gfx_device_inner }
    }

    /// Get screen dimensions.
    pub fn get_dimensions(&self) -> Option<(u32, u32)> {
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window, .. } => window.get_inner_size(),
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => None,
        }
    }

    /// Render all `Entity`s with `Renderable` components in `World`.
    pub fn render_world(&mut self, world: &mut ecs::World, pipeline: &renderer::Pipeline) {
        use components::rendering::{MeshInner, TextureInner, Renderable};
        use components::transform::Transform;
        use world_resources::camera::Projection;
        use renderer::Fragment;
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref mut renderer,
                                     ref mut device,
                                     ref window,
                                     .. } => {
                let camera = world.read_resource::<world_resources::Camera>().clone();

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

                let mut scene = renderer::Scene::<gfx_device_gl::Resources>::new(camera);
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
                let lights = world.read::<renderer::Light>();
                scene.lights.extend(lights.iter());
                // Render the `Scene`.
                renderer.submit(pipeline, &scene, device);
                window.swap_buffers().unwrap();
                device.cleanup();
                // Function that creates `Fragment`s from `Renderable`, `Transform` pairs.
                fn unwrap_renderable(renderable: &Renderable, global_transform: &Transform) -> Option<Fragment<gfx_device_gl::Resources>> {
                    let (buffer, slice) = match renderable.mesh.mesh_inner {
                        MeshInner::OpenGL { ref buffer,
                                            ref slice } => (buffer.clone(), slice.clone()),
                        _ => return None,
                    };
                    let ka = match renderable.ka.texture_inner {
                        TextureInner::OpenGL { ref texture } => texture.clone(),
                        _ => return None,
                    };
                    let kd = match renderable.kd.texture_inner {
                        TextureInner::OpenGL { ref texture } => texture.clone(),
                        _ => return None,
                    };
                    Some(Fragment {
                        transform: global_transform.clone().into(),
                        buffer: buffer,
                        slice: slice,
                        ka: ka,
                        kd: kd,
                    })
                }
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => (),
        }
    }

    /// Poll events from `GfxDevice`.
    pub fn poll_events(&mut self) -> Vec<WindowEvent> {
        let mut events = vec![];
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window, .. } => {
                for event in window.poll_events() {
                    let event = WindowEvent::new(event);
                    events.push(event);
                }
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => {
                unimplemented!();
            }
            GfxDeviceInner::Null => (),
        }
        events
    }
}
