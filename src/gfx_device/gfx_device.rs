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
use components::event::EngineEvent;
use world_resources;
use gfx_device::gfx_device_inner::GfxDeviceInner;

/// This struct holds all the graphics resources (except `MainTarget`) required to render a scene.
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

    /// Render all `Renderable`, `Transform` pairs in `World`.
    pub fn render_world(&mut self, world: &mut ecs::World, pipeline: &renderer::Pipeline) {
        use components::rendering::{MeshInner, TextureInner, Renderable};
        use components::transform::Transform;
        use world_resources::camera::Projection;
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
                let renderables = world.read::<Renderable>();
                let global_transforms = world.read::<Transform>();
                for (renderable, global_transform) in (&renderables, &global_transforms).iter() {
                    let (buffer, slice) = match renderable.mesh.mesh_inner {
                        MeshInner::OpenGL { ref buffer,
                                            ref slice } => { (buffer.clone(), slice.clone()) },
                        _ => continue,
                        };
                    let ka = match renderable.ka.texture_inner {
                        TextureInner::OpenGL { ref texture } => texture.clone(),
                        _ => continue,
                    };
                    let kd = match renderable.kd.texture_inner {
                        TextureInner::OpenGL { ref texture } => texture.clone(),
                        _ => continue,
                    };
                    let fragment = renderer::Fragment {
                        transform: global_transform.clone().into(),
                        buffer: buffer,
                        slice: slice,
                        ka: ka,
                        kd: kd,
                    };
                    scene.fragments.push(fragment);
                }
                let lights = world.read::<renderer::Light>();
                for light in lights.iter() {
                    scene.lights.push(light.clone());
                }
                renderer.submit(pipeline, &scene, device);
                window.swap_buffers().unwrap();
            }
            #[cfg(windows)]
            GfxDeviceInner::Direct3D {} => unimplemented!(),
            GfxDeviceInner::Null => (),
        }
    }

    /// Poll events from `GfxDevice`.
    pub fn poll_events(&mut self) -> Vec<EngineEvent> {
        let mut events = vec![];
        match self.gfx_device_inner {
            GfxDeviceInner::OpenGL { ref window, .. } => {
                for event in window.poll_events() {
                    let event = EngineEvent::new(event);
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
