use ecs::{Processor, RunArg, Join, Component, VecStorage};
use context::Context;
use std::sync::{Mutex, Arc};
use renderer;
use std::collections::HashSet;

pub struct RenderingProcessor {
    scene_name: String,
}

impl RenderingProcessor {
    pub fn new(scene_name: &str, context: &mut Context) -> RenderingProcessor {
        context.renderer.add_scene(scene_name);
        RenderingProcessor {
            scene_name: scene_name.into(),
        }
    }
}

unsafe impl Send for RenderingProcessor {  }

impl Processor<Arc<Mutex<Context>>> for RenderingProcessor {
    fn run(&mut self, arg: RunArg, context: Arc<Mutex<Context>>) {
        if let Ok(mut context) = context.lock() {
            let (entities, mut renderables, mut lights) = arg.fetch(|w| (w.entities(), w.write::<Renderable>(), w.write::<Light>()));

            let mut light_indices = HashSet::<usize>::new();
            for (entity, light) in (&entities, &mut lights).iter() {
                match light.idx {
                    Some(idx) => {
                        // If this Light is already in frame then update it.
                        light_indices.insert(idx);
                        let scene_name = self.scene_name.as_str();
                        if let Some(frame_light) = context.renderer.mut_light(scene_name, idx) {
                            *frame_light = light.light.clone();
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because RenderingProcessor::scene_name field is invalid \
                                      or Light::idx field is invalid.", entity.get_id());
                            arg.delete(entity);
                        }
                    },
                    None => {
                        // Otherwise add it to the frame.
                        let scene_name = self.scene_name.as_str();
                        let frame_light = light.light.clone();
                        if let Some(idx) = context.renderer.add_light(scene_name, frame_light) {
                            // If this Light can be added to the frame then add it and store
                            // the index in the light.idx field.
                            light.idx = Some(idx);
                            light_indices.insert(idx);
                        } else {
                            // Otherwise log an error and delete this entity.
                            // TODO: Implement proper logging
                            println!("Error: entity with id = {0} is deleted, \
                                      because RenderingProcessor::scene_name field is invalid.", entity.get_id());
                            arg.delete(entity);
                        }
                    }
                }
            }

            let mut renderable_indices = HashSet::<usize>::new();
            for (entity, renderable) in (&entities, &mut renderables).iter() {
                match renderable.idx {
                    // If this Renderable is already in frame then update the transform field
                    // of the corresponding Fragment.
                    Some(idx) => {
                        renderable_indices.insert(idx);
                        let scene_name = self.scene_name.as_str();
                        if let Some(transform) = context.renderer.mut_fragment_transform(scene_name, idx) {
                            *transform = renderable.transform;
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because RenderingProcessor::scene_name field is invalid \
                                      or Renderable::idx field is invalid.", entity.get_id());
                            arg.delete(entity);
                        }
                    },
                    // If it is not in frame then attempt to create a Fragment with given transform
                    // and requested mesh, ka, and kd, which are looked up using the asset manager
                    None => {
                        let scene_name = self.scene_name.as_str();
                        let mesh = renderable.mesh.as_str();
                        let ka = renderable.ka.as_str();
                        let kd = renderable.kd.as_str();
                        let transform = renderable.transform;
                        if let Some(fragment) = context.asset_manager.get_fragment(mesh, ka, kd, transform) {
                            if let Some(idx) = context.renderer.add_fragment(scene_name, fragment) {
                                // If this Renderable can be added to the frame then add it and store
                                // the index of this fragment in the renderable.idx field
                                renderable.idx = Some(idx);
                                renderable_indices.insert(idx);
                            } else {
                                // Otherwise log an error and delete this entity
                                println!("Error: entity with id = {0} is deleted, \
                                          because RenderingProcessor::scene_name field is invalid.", entity.get_id());
                                arg.delete(entity);
                            }
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because at least one of the assets requested by the Renderable \
                                      component attached to this entity doesn't exist.", entity.get_id());
                            arg.delete(entity);
                        }
                    }
                }
            }

            // Delete from frame all renderer::Lights corresponding to deleted Light components
            if let Some(num_lights) = context.renderer.num_lights(self.scene_name.as_str()) {
                for i in 0..num_lights {
                    if !light_indices.contains(&i) {
                        context.renderer.delete_light(self.scene_name.as_str(), i);
                    }
                }
            }

            // Delete from frame all Fragments corresponding to deleted Renderable components
            if let Some(num_fragments) = context.renderer.num_fragments(self.scene_name.as_str()) {
                for i in 0..num_fragments {
                    if !renderable_indices.contains(&i) {
                        context.renderer.delete_fragment(self.scene_name.as_str(), i);
                    }
                }
            }
        }
    }
}

/// Entities with this component are rendered
/// by the `RenderingProcessor`, modyfing the `transform` field
/// would affect the `transform` of the `Fragment` that is
/// being rendered.
pub struct Renderable {
    // This field holds the index which can be used
    // to access the renderer::Fragment held by context.renderer
    // If idx == None then this Renderable is not renderered.
    idx: Option<usize>,
    mesh: String,
    ka: String,
    kd: String,
    pub transform: [[f32; 4]; 4],
}

impl Renderable {
    /// Create a new Renderable component from scene_name
    /// and names of assets loaded by context.asset_manager.
    pub fn new(mesh: &str, ka: &str, kd: &str, transform: [[f32; 4]; 4]) -> Renderable {
        Renderable {
            idx: None,
            mesh: mesh.into(),
            ka: ka.into(),
            kd: kd.into(),
            transform: transform,
        }
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Renderable>;
}

/// A light component.
/// All changes in the Light::light field will be
/// applied to the associated light in the frame.
pub struct Light {
    // This field holds the index which can be used to access the renderer::Light
    // held by context.renderer.
    // If idx == None then this Light doesn't affect the rendered image.
    idx: Option<usize>,
    pub light: renderer::Light,
}

impl Light {
    // Create a new light from scene_name and a renderer::Light.
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
