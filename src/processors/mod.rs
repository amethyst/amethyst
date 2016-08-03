use ecs::{Processor, RunArg, Join, Component, VecStorage};
use context::Context;
use std::sync::{Mutex, Arc};

pub struct RenderingProcessor;

unsafe impl Send for RenderingProcessor {  }

impl Processor<Arc<Mutex<Context>>> for RenderingProcessor {
    fn run(&mut self, arg: RunArg, context: Arc<Mutex<Context>>) {
        if let Ok(mut context) = context.lock() {
            let (entities, mut renderables) = arg.fetch(|w| (w.entities(), w.write::<Renderable>()));
            for (entity, renderable) in (&entities, &mut renderables).iter() {
                match renderable.idx {
                    // If this Renderable is already in frame then update the transform field
                    // of the corresponding Fragment.
                    Some(idx) => {
                        let scene_name = renderable.scene_name.as_str();
                        if let Some(transform) = context.renderer.mut_fragment_transform(scene_name, idx) {
                            *transform = renderable.transform;
                        } else {
                            println!("Error: entity with id = {0} is deleted, \
                                      because Renderable::scene_name field is invalid \
                                      or Renderable::idx field is invalid.", entity.get_id());
                            arg.delete(entity);
                        }
                    },
                    // If it is not in frame then attempt to create a Fragment with given transform
                    // and requested mesh, ka, and kd, which are looked up using the asset manager
                    None => {
                        let scene_name = renderable.scene_name.as_str();
                        let mesh = renderable.mesh.as_str();
                        let ka = renderable.ka.as_str();
                        let kd = renderable.kd.as_str();
                        let transform = renderable.transform;
                        if let Some(fragment) = context.asset_manager.get_fragment(mesh, ka, kd, transform) {
                            if let Some(idx) = context.renderer.add_fragment(scene_name, fragment) {
                                // If this Renderable can be added to the frame then add it and store
                                // the index of this fragment in the renderable.idx field
                                renderable.idx = Some(idx);
                            } else {
                                // Otherwise log an error and delete this entity
                                // TODO: Implement proper logging;
                                println!("Error: entity with id = {0} is deleted, \
                                          because Renderable::scene_name field is invalid.", entity.get_id());
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
        }
    }
}


/// Entities with this component are rendered
/// by the `RenderingProcessor`, modyfing the `transform` field
/// would affect the `transform` of the `Fragment` that is
/// being rendered.
pub struct Renderable {
    idx: Option<usize>,
    scene_name: String,
    mesh: String,
    ka: String,
    kd: String,
    pub transform: [[f32; 4]; 4],
}

impl Renderable {
    pub fn new(scene_name: &str, mesh: &str, ka: &str, kd: &str, transform: [[f32; 4]; 4]) -> Renderable {
        Renderable {
            idx: None,
            scene_name: scene_name.into(),
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
