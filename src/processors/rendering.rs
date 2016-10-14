//! Default rendering processor types.

extern crate cgmath;

use ecs::{Processor, RunArg, Join, Component, VecStorage, Entity};
use context::Context;
use std::sync::{Mutex, Arc};
use renderer;
use context::prefab::PrefabIndex;
use renderer::Layer;
use std::collections::HashSet;
use config::Element;
use std::path::Path;

use context::resource::{MeshID, TextureID};

config!(
/// A config required to create a rendering processor.
    struct RendererConfig {
// Forward or Deferred
        pub pipeline: String = "Forward".to_string(),
// Flat or Shaded
        pub shading: String = "Flat".to_string(),
        pub clear_color: [f32; 4] = [0., 0., 0., 1.],
    }
);

/// A rendering processor struct.
pub struct RenderingProcessor {
    active_camera: Option<Entity>,
}

const ACTIVE_CAMERA_NAME: &'static str = "main";
const ACTIVE_SCENE_NAME: &'static str = "main";

fn forward_flat(clear_color: [f32; 4]) -> Vec<Layer> {
    use renderer::pass::*;

    vec![
        Layer::new("main",
            vec![
                Clear::new(clear_color),
                DrawFlat::new(ACTIVE_CAMERA_NAME, ACTIVE_SCENE_NAME),
            ]
        ),
    ]
}

fn forward_shaded(clear_color: [f32; 4]) -> Vec<Layer> {
    use renderer::pass::*;

    vec![
        Layer::new("main",
            vec![
                Clear::new(clear_color),
                DrawShaded::new(ACTIVE_CAMERA_NAME, ACTIVE_SCENE_NAME),
            ]
        ),
    ]
}

fn layer_gbuffer(clear_color: [f32; 4]) -> Layer {
    use renderer::pass::*;

    Layer::new("gbuffer",
               vec![
            Clear::new(clear_color),
            DrawFlat::new(ACTIVE_CAMERA_NAME, ACTIVE_SCENE_NAME),
        ])
}

fn deferred_flat(clear_color: [f32; 4]) -> Vec<Layer> {
    use renderer::pass::*;

    vec![
        layer_gbuffer(clear_color),
        Layer::new("main",
            vec![
                BlitLayer::new("gbuffer", "ka"),
            ]
        ),
    ]
}

fn deferred_shaded(clear_color: [f32; 4]) -> Vec<Layer> {
    use renderer::pass::*;

    vec![
        layer_gbuffer(clear_color),
        Layer::new("main",
            vec![
                BlitLayer::new("gbuffer", "ka"),
                Lighting::new(ACTIVE_CAMERA_NAME, "gbuffer", ACTIVE_SCENE_NAME)
            ]
        ),
    ]
}

impl RenderingProcessor {
    pub fn new(renderer_config: RendererConfig, context: &mut Context) -> RenderingProcessor {
        let clear_color = renderer_config.clear_color;
        let pipeline = match (renderer_config.pipeline.as_str(),
                              renderer_config.shading.as_str()) {
            ("Forward", "Flat") => forward_flat(clear_color),
            ("Forward", "Shaded") => forward_shaded(clear_color),
            ("Deferred", "Flat") => deferred_flat(clear_color),
            ("Deferred", "Shaded") => deferred_shaded(clear_color),
            _ => panic!("Error: Can't provide rendering pipeline requested in renderer_config"),
        };

        context.renderer.add_scene(ACTIVE_SCENE_NAME);

        let (w, h) = context.renderer.get_dimensions().unwrap();
        let proj = renderer::Camera::perspective(60.0, w as f32 / h as f32, 1.0, 100.0);
        let eye = [0., 0., 0.];
        let target = [0., 0., 0.];
        let up = [0., 0., 1.];
        let view = renderer::Camera::look_at(eye, target, up);
        let camera = renderer::Camera::new(proj, view);

        context.renderer.add_camera(camera, ACTIVE_CAMERA_NAME);
        context.renderer.set_pipeline(pipeline);
        RenderingProcessor { active_camera: None }
    }
}

unsafe impl Send for RenderingProcessor {}

impl Processor<Arc<Mutex<Context>>> for RenderingProcessor {
    fn run(&mut self, arg: RunArg, context: Arc<Mutex<Context>>) {
        if let Ok(mut context) = context.lock() {
            let (entities, mut renderables, mut lights, mut cameras) = arg.fetch(|w| {
                (w.entities(), w.write::<Renderable>(), w.write::<Light>(), w.write::<Camera>())
            });

            let mut light_indices = HashSet::<usize>::new();
            for (entity, light) in (&entities, &mut lights).iter() {
                match light.idx {
                    Some(idx) => {
                        // If this Light is already in frame then update it.
                        light_indices.insert(idx);
                        if let Some(frame_light) = context.renderer
                            .mut_light(ACTIVE_SCENE_NAME, idx) {
                            *frame_light = light.light.clone();
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because Light::idx field is invalid.", entity.get_id());
                            arg.delete(entity);
                        }
                    }
                    None => {
                        // Otherwise add it to the frame.
                        let frame_light = light.light.clone();
                        if let Some(idx) = context.renderer
                            .add_light(ACTIVE_SCENE_NAME, frame_light) {
                            // If this Light can be added to the frame then add it and store
                            // the index in the light.idx field.
                            light.idx = Some(idx);
                            light_indices.insert(idx);
                        }
                    }
                }
            }

            let mut renderable_indices = HashSet::<usize>::new();
            for (entity, renderable) in (&entities, &mut renderables).iter() {
                renderable.update_transform_matrix();
                match renderable.idx {
                    // If this Renderable is already in frame then update the transform field
                    // of the corresponding Fragment.
                    Some(idx) => {
                        renderable_indices.insert(idx);
                        if let Some(transform) = context.renderer
                            .mut_fragment_transform(ACTIVE_SCENE_NAME, idx) {
                            *transform = renderable.transform;
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because Renderable::idx field is invalid.", entity.get_id());
                            arg.delete(entity);
                        }
                    }
                    // If it is not in frame then attempt to create a Fragment with given transform
                    // and requested mesh, ka, and kd, which are looked up using the asset manager
                    None => {
                        let ref mesh = renderable.mesh;
                        let ref ka = renderable.ka;
                        let ref kd = renderable.kd;
                        let transform = renderable.transform;
                        if let Some(fragment) = context.prefab_manager
                            .get_fragment(mesh, ka, kd, transform) {
                            if let Some(idx) = context.renderer
                                .add_fragment(ACTIVE_SCENE_NAME, fragment) {
                                // If this Renderable can be added to
                                // the frame then add it and store
                                // the index of this fragment
                                // in the renderable.idx field
                                renderable.idx = Some(idx);
                                renderable_indices.insert(idx);
                            }
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because at least one of the assets \
                                      requested by the Renderable \
                                      component attached to this \
                                      entity doesn't exist.", entity.get_id());
                            arg.delete(entity);
                        }
                    }
                }
            }

            if let Some(active_camera) = self.active_camera {
                if let Some(camera) = cameras.get(active_camera) {
                    let proj = match camera.projection {
                        Projection::Perspective { fov, aspect, near, far } => {
                            renderer::Camera::perspective(fov, aspect, near, far)
                        }
                        Projection::Orthographic { left, right, bottom, top, near, far } => {
                            renderer::Camera::orthographic(left, right, bottom, top, near, far)
                        }
                    };

                    let eye = camera.eye;
                    let target = camera.target;
                    let up = camera.up;

                    let view = renderer::Camera::look_at(eye, target, up);

                    if let Some(camera) = context.renderer.mut_camera(ACTIVE_CAMERA_NAME) {
                        *camera = renderer::Camera::new(proj, view);
                    }
                } else {
                    self.active_camera = None;
                }
            }

            for (entity, camera) in (&entities, &mut cameras).iter() {
                if camera.activate {
                    self.active_camera = Some(entity);
                    camera.activate = false;
                }
            }

            // Delete from frame all renderer::Lights corresponding to deleted Light components
            if let Some(num_lights) = context.renderer.num_lights(ACTIVE_SCENE_NAME) {
                for i in 0..num_lights {
                    if !light_indices.contains(&i) {
                        context.renderer.delete_light(ACTIVE_SCENE_NAME, i);
                    }
                }
            }

            // Delete from frame all Fragments corresponding to deleted Renderable components
            if let Some(num_fragments) = context.renderer.num_fragments(ACTIVE_SCENE_NAME) {
                for i in 0..num_fragments {
                    if !renderable_indices.contains(&i) {
                        context.renderer.delete_fragment(ACTIVE_SCENE_NAME, i);
                    }
                }
            }
        }
    }
}

/// Entities with this component are rendered
/// by the `RenderingProcessor`, modifying the `transform` field
/// would affect the `transform` of the `Fragment` that is
/// being rendered.
#[derive(Clone)]
pub struct Renderable {
    // This field holds the index which can be used
    // to access the renderer::Fragment held by context.renderer
    // If idx == None then this Renderable is not renderered.
    idx: Option<usize>,
    mesh: MeshID,
    ka: TextureID,
    kd: TextureID,
    transform: [[f32; 4]; 4],
    pub translation: [f32; 3],
    pub rotation_axis: [f32; 3],
    pub rotation_angle: f32,
    pub scale: [f32; 3],
}

impl Renderable {
    /// Create a new Renderable component from names of assets loaded by context.resource_manager.
    pub fn new(mesh: MeshID, ka: TextureID, kd: TextureID) -> Renderable {
        Renderable {
            idx: None,
            mesh: mesh,
            ka: ka,
            kd: kd,
            transform: cgmath::Matrix4::from_scale(1.).into(),
            translation: [0., 0., 0.],
            rotation_axis: [0., 0., 0.],
            rotation_angle: 0.,
            scale: [1., 1., 1.],
        }
    }

    fn update_transform_matrix(&mut self) {
        let translation = self.translation;
        let translation = cgmath::Vector3::new(translation[0], translation[1], translation[2]);
        let translation_matrix = cgmath::Matrix4::from_translation(translation);
        let rotation_axis = self.rotation_axis;
        let rotation_axis =
            cgmath::Vector3::new(rotation_axis[0], rotation_axis[1], rotation_axis[2]);
        let rotation_angle = self.rotation_angle;
        let rotation_angle = cgmath::Rad(rotation_angle);
        let rotation_matrix = cgmath::Matrix4::from_axis_angle(rotation_axis, rotation_angle);
        let scale = self.scale;
        let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
        let transform = translation_matrix * rotation_matrix * scale_matrix;
        self.transform = transform.into();
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}

/// A `Light` component.
/// All changes in the `light` field will be
/// applied to the associated `renderer::Light` in the frame.
#[derive(Copy, Clone)]
pub struct Light {
    // This field holds the index which can be used to access the renderer::Light
    // held by context.renderer.
    // If idx == None then this Light doesn't affect the rendered image.
    idx: Option<usize>,
    pub light: renderer::Light,
}

impl Light {
    // Create a new `Light` component from a `renderer::Light`.
    pub fn new(light: renderer::Light) -> Light {
        Light {
            idx: None,
            light: light,
        }
    }
}

impl Component for Light {
    type Storage = VecStorage<Light>;
}

/// A projection enum which is required to create a `Camera` component.
#[derive(Copy, Clone)]
pub enum Projection {
    Perspective {
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

/// A `Camera` component.
/// If this `Camera` is active then all changes in this component's fields
/// will be applied to the camera that is being used to render the scene.
#[derive(Copy, Clone)]
pub struct Camera {
    pub projection: Projection,
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    activate: bool,
}

impl Camera {
    /// Create a new `Camera` component from all the parameters
    /// for projection and view transformations.
    pub fn new(projection: Projection, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Camera {
        Camera {
            projection: projection,
            eye: eye,
            target: target,
            up: up,
            activate: false,
        }
    }

    // Note: If this method is called more than once per frame, then
    // the Camera that was created first will be activated, not the one
    // that called this method last.
    pub fn activate(&mut self) {
        self.activate = true;
    }
}

impl Component for Camera {
    type Storage = VecStorage<Camera>;
}
