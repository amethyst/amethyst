
extern crate cgmath;
// extern crate test;

use self::cgmath::{Quaternion, Vector3, Matrix3, Matrix4};

use ecs::{Join, Component, NullStorage, VecStorage, Entity, RunArg, Processor};
use context::Context;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::{HashMap, HashSet};

/// Local position and rotation from parent.
#[derive(Debug)]
pub struct LocalTransform {
    pos: [f32; 3], // translation vector
    rot: [f32; 4], // quaternion [w (scalar), x, y, z]
    scale: [f32; 3], // scale vector
    dirty: AtomicBool,
}

impl LocalTransform {
    #[inline]
    pub fn pos(&self) -> [f32; 3] {
        self.pos
    }
    #[inline]
    pub fn rot(&self) -> [f32; 4] {
        self.rot
    }
    #[inline]
    pub fn scale(&self) -> [f32; 3] {
        self.scale
    }
    #[inline]
    pub fn set_pos(&mut self, pos: [f32; 3]) {
        self.pos = pos;
        self.dirty.store(true, Ordering::SeqCst);
    }
    #[inline]
    pub fn set_rot(&mut self, rot: [f32; 4]) {
        self.rot = rot;
        self.dirty.store(true, Ordering::SeqCst);
    }
    #[inline]
    pub fn set_scale(&mut self, scale: [f32; 3]) {
        self.scale = scale;
        self.dirty.store(true, Ordering::SeqCst);
    }
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let quat: Matrix3<f32> = Quaternion::from(self.rot).into();
        let mut matrix: Matrix4<f32> = (&quat *
                                        Matrix3::new(self.scale[0],
                                                     0.0,
                                                     0.0,
                                                     0.0,
                                                     self.scale[1],
                                                     0.0,
                                                     0.0,
                                                     0.0,
                                                     self.scale[2]))
            .into();
        matrix.w = Vector3::from(self.pos).extend(1.0f32);
        matrix.into()
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        LocalTransform {
            pos: [0.0, 0.0, 0.0],
            rot: [1.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            dirty: AtomicBool::new(true),
        }
    }
}

impl Component for LocalTransform {
    type Storage = VecStorage<LocalTransform>;
}

/// Absolute transformation (transformed from origin)
/// Should be used for rendering position and orientation.
#[derive(Debug, Copy, Clone)]
pub struct Transform(pub [[f32; 4]; 4]);
impl Transform {
    pub fn identity() -> Self {
        Transform([[1.0, 0.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]])
    }
}

impl Component for Transform {
    type Storage = VecStorage<Transform>;
}

/// Initialization component
/// Added to entity with a `LocalTransform` component after the first update.
#[derive(Default, Copy, Clone)]
pub struct Init;
impl Component for Init {
    type Storage = NullStorage<Init>;
}

pub struct Parent {
    parent: Entity,
    dirty: AtomicBool,
}

impl Parent {
    pub fn new(entity: Entity) -> Parent {
        Parent {
            parent: entity,
            dirty: AtomicBool::new(true),
        }
    }

    #[inline]
    pub fn parent(&self) -> Entity {
        self.parent
    }
    #[inline]
    pub fn set_parent(&mut self, entity: Entity) {
        self.parent = entity;
        self.dirty.store(true, Ordering::SeqCst);
    }
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

impl Component for Parent {
    type Storage = VecStorage<Parent>;
}

/// Transformation processor.
/// Handles updating `Transform` components based on the `LocalTransform` component and parents.
#[derive(Debug)]
pub struct TransformProcessor {
    // Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,

    // Vec of entities with parents before children. Only contains entities with parents.
    sorted: Vec<(Entity, Entity)>,

    // New entities in current update
    new: Vec<Entity>,

    // Entities that have been removed in current frame.
    dead: HashSet<Entity>,

    // Parent entities that were dirty.
    dirty: HashSet<Entity>,
}

impl TransformProcessor {
    pub fn new() -> TransformProcessor {
        TransformProcessor {
            indices: HashMap::new(),
            sorted: Vec::new(),
            new: Vec::new(),
            dead: HashSet::new(),
            dirty: HashSet::new(),
        }
    }
}

impl Processor<Arc<Mutex<Context>>> for TransformProcessor {
    fn run(&mut self, arg: RunArg, _: Arc<Mutex<Context>>) {
        // Fetch world and gets entities/components
        let (entities, locals, mut globals, mut init, parents) = arg.fetch(|w| {
            let entities = w.entities();
            let locals = w.read::<LocalTransform>();
            let parents = w.read::<Parent>();

            // Deletes entities whose parents aren't alive.
            for (entity, _, parent) in (&entities, &locals, &parents).iter() {
                if self.dead.contains(&parent.parent()) || !w.is_alive(parent.parent()) {
                    arg.delete(entity);
                    self.dead.insert(entity);
                }
            }

            (entities, locals, w.write::<Transform>(), w.write::<Init>(), parents)
        });

        // Checks for entities with a local transform and parent, but no `Init` component.
        for (entity, _, parent, _) in (&entities, &locals, &parents, !&init).iter() {
            self.indices.insert(entity, self.sorted.len());
            self.sorted.push((entity, parent.parent()));
            self.new.push(entity.clone());
        }

        // Adds an `Init` component to the entity.
        for entity in self.new.drain(..) {
            init.insert(entity, Init);
        }

        // Compute transforms without parents.
        for (local, global, _) in (&locals, &mut globals, !&parents).iter() {
            if local.is_dirty() {
                global.0 = local.matrix();
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let (entity, parent_entity) = self.sorted[index];
            let mut swap = None;

            match parents.get(entity) {
                Some(parent) => {
                    if let Some(local) = locals.get(entity) {
                        // Check if parent is alive.
                        // Make sure the transform is also dirty if the parent has changed.
                        if parent.is_dirty() {
                            // If the index is none then the parent is likely an orphan or dead
                            if let Some(parent_index) = self.indices.get(&parent_entity) {
                                let parent_index = parent_index.clone();
                                if parent_index > index {
                                    swap = Some((index, parent_index));
                                }
                            }

                            if let Some((i, p)) = swap {
                                // Swap the parent and child.
                                self.sorted.swap(p, i);
                                self.indices.insert(parent_entity, i);
                                self.indices.insert(entity, p);

                                // Swap took place, re-try this index.
                                continue;
                            }

                            local.dirty.store(true, Ordering::SeqCst);
                        }

                        if local.is_dirty() || self.dirty.contains(&parent_entity) {
                            let combined_transform = if let Some(parent_global) =
                                                            globals.get(parent_entity) {
                                Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())
                            } else {
                                Matrix4::from(local.matrix())
                            };

                            if let Some(global) = globals.get_mut(entity) {
                                global.0 = combined_transform.into();
                            }

                            local.dirty.store(false, Ordering::SeqCst);
                            parent.dirty.store(false, Ordering::SeqCst);
                            self.dirty.insert(entity);
                        }
                    }
                }
                None => {
                    // Parent component was removed.
                    // Therefore, remove from sorted and indices.
                    self.sorted.swap_remove(index); // swap with last to prevent shift
                    if let Some(swapped) = self.sorted.get(index) {
                        self.indices.insert(swapped.0, index);

                        // Make sure to check for parent swap next iteration
                        if let Some(parent) = parents.get(swapped.0) {
                            parent.dirty.store(true, Ordering::SeqCst);
                        }
                    }
                    self.indices.remove(&entity);
                    init.remove(entity);

                    // Re-try index because swapped with last element.
                    continue;
                }
            }

            index += 1;
        }

        self.dirty.clear();
    }
}

#[cfg(test)]
mod tests {
    // use super::test::Bencher;
    use super::*;
    use super::cgmath::{Decomposed, Quaternion, Vector3, Matrix4};
    use ecs::{Planner, World, RunArg};
    use engine::Config;
    use context::Context;
    use std::sync::{Arc, Mutex};

    #[test]
    fn transform_matrix() {
        let mut transform = LocalTransform::default();
        transform.set_pos([5.0, 2.0, -0.5]);
        transform.set_rot([1.0, 0.0, 0.0, 0.0]);
        transform.set_scale([2.0, 2.0, 2.0]);

        let decomposed = Decomposed {
            rot: Quaternion::from(transform.rot),
            disp: Vector3::from(transform.pos),
            scale: 2.0,
        };

        let matrix = transform.matrix();
        let cg_matrix: Matrix4<f32> = decomposed.into();
        let cg_matrix: [[f32; 4]; 4] = cg_matrix.into();

        assert_eq!(matrix, cg_matrix);
    }

    #[test]
    fn transform_processor() {
        let config = Config::default();
        let ctx = Arc::new(Mutex::new(Context::new(config.context_config)));
        let mut world = World::new();

        world.register::<LocalTransform>();
        world.register::<Transform>();
        world.register::<Init>();
        world.register::<Parent>();

        // test whether deleting the parent deletes the child
        let e1 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::identity())
            .build();

        let e2 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Parent>(Parent::new(e1))
            .with::<Transform>(Transform::identity())
            .build();

        // test whether deleting an entity deletes the child
        let e3 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::identity())
            .build();

        let e4 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Parent>(Parent::new(e3))
            .with::<Transform>(Transform::identity())
            .build();

        // let e5 = world.create_now()
        // .with::<LocalTransform>(LocalTransform::default())
        // .with::<Parent>(Parent::new(e4))
        // .with::<Transform>(Transform::identity())
        // .build();

        let mut planner: Planner<Arc<Mutex<Context>>> = Planner::new(world, 1);
        let transform_processor = TransformProcessor::new();
        planner.add_system::<TransformProcessor>(transform_processor, "transform_processor", 0);

        {
            let mut world = planner.mut_world();
            // world.delete_now(e1);

            let mut parents = world.write::<Parent>();
            // parents.remove(e2);
        }

        // frame 1
        planner.dispatch(ctx.clone());
        planner.wait();

        {
            let mut world = planner.mut_world();
            // world.delete_now(e1);

            let mut parents = world.write::<Parent>();
            // parents.insert(e2
            parents.remove(e2);
        }

        // frame 2
        planner.dispatch(ctx.clone());
        planner.wait();

        {
            let world = planner.mut_world();

            let mut parents = world.write::<Parent>();
            parents.insert(e2, Parent::new(e1));
        }

        // frame 3
        planner.dispatch(ctx.clone());
        planner.wait();
    }


    // Add #![feature(test)] to use.
    // #[bench]
    // fn bench_processor(b: &mut Bencher) {
    // let mut world = World::new();
    //
    // world.register::<LocalTransform>();
    // world.register::<Transform>();
    // world.register::<Init>();
    // world.register::<Parent>();
    //
    // let mut prev_entity = world.create_now()
    // .with::<LocalTransform>(LocalTransform::default())
    // .with::<Transform>(Transform::identity())
    // .build();
    //
    // for i in 0..50_000 {
    // let mut transform = LocalTransform::default();
    //
    // if i % 5 == 0 {
    // prev_entity = world.create_now()
    // .with::<LocalTransform>(transform)
    // .with::<Parent>(Parent::new(prev_entity))
    // .with::<Transform>(Transform::identity())
    // .build();
    // } else {
    // prev_entity = world.create_now()
    // .with::<LocalTransform>(transform)
    // .with::<Transform>(Transform::identity())
    // .build();
    // }
    // }
    //
    // let mut planner: Planner<Arc<Mutex<Context>>> = Planner::new(world, 1);
    // let transform_processor = TransformProcessor::new();
    // planner.add_system::<TransformProcessor>(transform_processor, "transform_processor", 0);
    //
    // let config = Config::default();
    // let ctx = Arc::new(Mutex::new(Context::new(config.context_config)));
    //
    // b.iter(|| {
    // planner.dispatch(ctx.clone());
    // });
    // }


}
