
extern crate cgmath;
// extern crate test;

use self::cgmath::{Quaternion, Vector3, Matrix3, Matrix4};

use ecs::{Join, Component, NullStorage, VecStorage, Entity, RunArg, Processor};
use context::Context;
use std::sync::{Mutex, Arc};
use std::collections::{HashMap, HashSet};

/// Local position and rotation from parent.
#[derive(Debug, Clone)]
pub struct LocalTransform {
    pos: [f32; 3], // translation vector
    rot: [f32; 4], // quaternion [w (scalar), x, y, z]
    scale: [f32; 3], // scale vector
    parent: Option<Entity>,
    dirty: bool,
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
    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }
    #[inline]
    pub fn set_pos(&mut self, pos: [f32; 3]) {
        self.pos = pos;
        self.dirty = true;
    }
    #[inline]
    pub fn set_rot(&mut self, rot: [f32; 4]) {
        self.rot = rot;
        self.dirty = true;
    }
    #[inline]
    pub fn set_scale(&mut self, scale: [f32; 3]) {
        self.scale = scale;
        self.dirty = true;
    }
    #[inline]
    pub fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = parent;
        self.dirty = true;
    }
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
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
            parent: None,
            dirty: true,
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

/// Transformation processor.
/// Handles updating `Transform` components based on the `LocalTransform` component and parents.
pub struct TransformProcessor {
    // Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,

    // Vec of entities with parents before children.
    sorted: Vec<(Entity, Option<Entity>)>,

    // Possible candidates for swapping parents and children in sorted.
    // Also candidates for parent transforms being removed.
    candidates: Vec<(Entity, Entity)>,

    // New entities in current update
    new: Vec<Entity>,

    // Entities that have removed their transforms.
    removed: HashSet<Entity>,
}

impl TransformProcessor {
    pub fn new() -> TransformProcessor {
        TransformProcessor {
            indices: HashMap::new(),
            sorted: Vec::new(),
            candidates: Vec::new(),
            new: Vec::new(),
            removed: HashSet::new(),
        }
    }
}

impl Processor<Arc<Mutex<Context>>> for TransformProcessor {
    fn run(&mut self, arg: RunArg, _: Arc<Mutex<Context>>) {
        // Fetch world and gets entities/components
        let (entities, mut locals, mut globals, mut init) = arg.fetch(|w| {
            let entities = w.entities();
            let locals = w.write::<LocalTransform>();

            // Deletes entities whose parents aren't alive.
            for (entity, local) in (&entities, &locals).iter() {
                if let Some(parent) = local.parent {
                    if !w.is_alive(parent) {
                        arg.delete(entity);
                    }
                }
            }

            (entities, locals, w.write::<Transform>(), w.write::<Init>())
        });

        // Checks for entities with a local transform, but no `Init` component.
        for (entity, local, _) in (&entities, &locals, !&init).iter() {
            self.indices.insert(entity, self.sorted.len());
            self.sorted.push((entity, local.parent));
            self.new.push(entity.clone());
        }

        // Adds an `Init` component to the entity
        for entity in self.new.drain(..) {
            init.insert(entity, Init);
        }

        // Compute transforms (global) from local transforms and parents.
        for &(entity, parent_option) in self.sorted.iter() {
            match locals.get(entity) {
                Some(local) => {
                    let combined_transform = match parent_option {
                        Some(parent) => {
                            self.candidates.push((entity, parent));

                            if let Some(parent_global) = globals.get(parent) {
                                Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())
                            } else {
                                Matrix4::from(local.matrix())
                            }
                        }
                        None => Matrix4::from(local.matrix()),
                    };

                    if let Some(global) = globals.get_mut(entity) {
                        global.0 = combined_transform.into();
                    }
                }
                None => {
                    // Local Transform is deleted/removed
                    // Add to removed set so that child can remove their parent
                    self.removed.insert(entity);
                }
            }
        }

        // Checks whether the child is before the parent.
        // If so, it swaps their positions.
        for (entity, parent) in self.candidates.drain(..) {
            // Check if parent transform is removed.
            if self.removed.contains(&parent) {
                if let Some(local) = locals.get_mut(entity) {
                    local.parent = None;
                }
                continue;
            }

            let parent_index: usize = self.indices.get(&parent).unwrap().clone();
            let index: usize = self.indices.get(&entity).unwrap().clone();
            if parent_index > index {
                self.sorted.swap(parent_index, index);
                self.indices.insert(parent, index);
                self.indices.insert(entity, parent_index);
            }
        }

        // Remove entities who no longer have transforms from sorted
        for entity in self.removed.drain() {
            let index = self.indices.get(&entity).unwrap().clone();
            self.sorted.swap_remove(index);
            if let Some(swapped) = self.sorted.get(index) {
                self.indices.insert(swapped.0, index);
            }
            self.indices.remove(&entity);
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::test::Bencher;
    use super::*;
    use super::cgmath::{Decomposed, Quaternion, Vector3, Matrix4};
    use ecs::{Planner, World};
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

        // test whether deleting the parent deletes the child
        let e1 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::identity())
            .build();

        let mut t2 = LocalTransform::default();
        t2.parent = Some(e1);

        let e2 = world.create_now()
            .with::<LocalTransform>(t2)
            .with::<Transform>(Transform::identity())
            .build();

        // test whether deleting an entity deletes the child
        let e3 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::identity())
            .build();

        let mut t4 = LocalTransform::default();
        t4.parent = Some(e3);

        let e4 = world.create_now()
            .with::<LocalTransform>(t4)
            .with::<Transform>(Transform::identity())
            .build();

        let mut planner: Planner<Arc<Mutex<Context>>> = Planner::new(world, 1);
        let transform_processor = TransformProcessor::new();
        planner.add_system::<TransformProcessor>(transform_processor, "transform_processor", 0);

        // frame 1
        planner.dispatch(ctx.clone());
        planner.wait();

        {
            let mut world = planner.mut_world();
            world.delete_now(e3);

            let mut locals = world.write::<LocalTransform>();
            locals.remove(e1);
        }

        // frame 2
        planner.dispatch(ctx.clone());
        planner.wait();

        {
            let world = planner.mut_world();
            assert_eq!(world.is_alive(e3), false);
            assert_eq!(world.is_alive(e4), false);

            let locals = world.read::<LocalTransform>();
            assert_eq!(locals.get(e2).unwrap().parent, None);
        }
    }


    // Add #![feature(test)] to use.
    // #[bench]
    // fn bench_processor(b: &mut Bencher) {
    // let mut world = World::new();
    //
    // world.register::<LocalTransform>();
    // world.register::<Transform>();
    // world.register::<Init>();
    //
    // let mut prev_entity = world.create_now()
    // .with::<LocalTransform>(LocalTransform::default())
    // .with::<Transform>(Transform::identity())
    // .build();
    //
    // for i in 0..50_000 {
    // let mut transform = LocalTransform::default();
    //
    // prev_entity = world.create_now()
    // .with::<LocalTransform>(transform)
    // .with::<Transform>(Transform::identity())
    // .build();
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
