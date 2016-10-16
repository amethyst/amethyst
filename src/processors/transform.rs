//! Scene graph processor and types

extern crate cgmath;
// extern crate test;

use self::cgmath::{Quaternion, Vector3, Matrix3, Matrix4};

use ecs::{Join, Component, NullStorage, VecStorage, Entity, RunArg, Processor};
use context::Context;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::{HashMap, HashSet};

/// Local position, rotation, and scale (from parent if exists).
#[derive(Debug)]
pub struct LocalTransform {
    /// Translation vector [x, y, z]
    pos: [f32; 3],

    /// Quaternion [w (scalar), x, y, z]
    rot: [f32; 4],

    /// Scale vector [x, y, z]
    scale: [f32; 3],

    /// Flag for re-computation
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
        self.flag();
    }
    #[inline]
    pub fn set_rot(&mut self, rot: [f32; 4]) {
        self.rot = rot;
        self.flag();
    }
    #[inline]
    pub fn set_scale(&mut self, scale: [f32; 3]) {
        self.scale = scale;
        self.flag();
    }

    /// Set a specific part of the position without modifying the others
    /// (must be an index less than 3).
    /// Format: [0 = x, 1 = y, 2 = z]
    /// e.g. `transform.set_pos_index(1, 5.0)` // sets `y` to `5.0`
    #[inline]
    pub fn set_pos_index(&mut self, index: usize, val: f32) {
        if index < 3 {
            self.pos[index] = val;
        }
        self.flag();
    }

    /// Set a specific part of the rotation quaternion without modifying the others
    /// (must be an index less than 4).
    /// Format: [0 = w, 1 = x, 2 = y, 3 = z]
    /// e.g. `transform.set_rot_index(1, 0.0)` // sets `x` to `0.0`
    #[inline]
    pub fn set_rot_index(&mut self, index: usize, val: f32) {
        if index < 4 {
            self.rot[index] = val;
        }
        self.flag();
    }

    /// Set a specific part of the scale without modifying the others
    /// (must be an index less than 3).
    /// Format: [0 = x, 1 = y, 2 = z]
    /// e.g. `transform.set_scale_index(2, 3.0)` // sets `z` to `3.0`
    #[inline]
    pub fn set_scale_index(&mut self, index: usize, val: f32) {
        if index < 3 {
            self.scale[index] = val;
        }
        self.flag();
    }

    /// Flags the current transform for re-computation.
    #[inline]
    pub fn flag(&self) {
        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Returns whether or not the current transform is flagged for re-computation or "dirty".
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    /// Returns the local object matrix for the transform.
    /// Combined with the parent's global `Transform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let quat: Matrix3<f32> = Quaternion::from(self.rot).into();
        let scale: Matrix3<f32> = Matrix3::<f32> {
            x: [self.scale[0], 0.0, 0.0].into(),
            y: [0.0, self.scale[1], 0.0].into(),
            z: [0.0, 0.0, self.scale[2]].into(),
        };
        let mut matrix: Matrix4<f32> = (&quat * scale).into();
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

impl Component for Transform {
    type Storage = VecStorage<Transform>;
}

impl Default for Transform {
    fn default() -> Self {
        Transform([[1.0, 0.0, 0.0, 0.0],
                   [0.0, 1.0, 0.0, 0.0],
                   [0.0, 0.0, 1.0, 0.0],
                   [0.0, 0.0, 0.0, 1.0]])
    }
}

/// Initialization component
/// Added to entity with a `LocalTransform` component after the first update.
#[derive(Default, Copy, Clone)]
pub struct Init;
impl Component for Init {
    type Storage = NullStorage<Init>;
}

/// Component for defining a parent entity.
pub struct Parent {
    /// The parent entity
    parent: Entity,

    /// Flag for whether the parent was changed
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
                if !w.is_alive(parent.parent) || self.dead.contains(&parent.parent) {
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
                local.dirty.store(false, Ordering::SeqCst);
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let (entity, parent_entity) = self.sorted[index];

            match (parents.get(entity), locals.get(entity), self.dead.contains(&entity)) {
                (Some(parent), Some(local), false) => {
                    // Make sure the transform is also dirty if the parent has changed.
                    if parent.is_dirty() {
                        if parent.parent != parent_entity {
                            self.sorted[index] = (entity, parent.parent);
                        }

                        let mut swap = None;

                        // If the index is none then the parent is an orphan or dead
                        if let Some(parent_index) = self.indices.get(&parent.parent) {
                            let parent_index = parent_index.clone();
                            if parent_index > index {
                                swap = Some((index, parent_index));
                            }
                        }

                        if let Some((i, p)) = swap {
                            // Swap the parent and child.
                            self.sorted.swap(p, i);
                            self.indices.insert(parent.parent, i);
                            self.indices.insert(entity, p);

                            // Swap took place, re-try this index.
                            continue;
                        }

                        local.dirty.store(true, Ordering::SeqCst);
                    }

                    if local.is_dirty() || self.dirty.contains(&parent.parent) {
                        let combined_transform = if let Some(parent_global) =
                                                        globals.get(parent.parent) {
                            (Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())).into()
                        } else {
                            local.matrix()
                        };

                        if let Some(global) = globals.get_mut(entity) {
                            global.0 = combined_transform;
                        }

                        local.dirty.store(false, Ordering::SeqCst);
                        parent.dirty.store(false, Ordering::SeqCst);
                        self.dirty.insert(entity);
                    }
                }
                _ => {
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
    use ecs::{Planner, World, RunArg, Entity, Generation, InsertResult};
    use engine::Config;
    use context::Context;
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::Ordering;

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
            .with::<Transform>(Transform::default())
            .build();

        let e2 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::default())
            .build();

        // test whether deleting an entity deletes the child
        let e3 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::default())
            .build();

        let e4 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::default())
            .build();

        let e5 = world.create_now()
            .with::<LocalTransform>(LocalTransform::default())
            .with::<Transform>(Transform::default())
            .build();

        let mut planner: Planner<Arc<Mutex<Context>>> = Planner::new(world, 1);
        let transform_processor = TransformProcessor::new();
        planner.add_system::<TransformProcessor>(transform_processor, "transform_processor", 0);

        {
            let mut world = planner.mut_world();
            // world.delete_now(e1);

            let mut parents = world.write::<Parent>();
            parents.insert(e2, Parent::new(e1));
            parents.insert(e3, Parent::new(e2));
            parents.insert(e4, Parent::new(e1));

        }

        // frame 1
        println!("\nFRAME 1:\n---");
        planner.dispatch(ctx.clone());
        planner.wait();

        {
            let mut world = planner.mut_world();
            world.delete_now(e3);

            {
                let mut parents = world.write::<Parent>();
            }

            {
                let mut locals = world.write::<LocalTransform>();
                // locals.remove(e3);
            }
        }

        // frame 2
        println!("\nFRAME 2:\n---");
        planner.dispatch(ctx.clone());
        planner.wait();

        // {
        // let world = planner.mut_world();
        // world.delete_now(e3);
        //
        // let mut parents = world.write::<Parent>();
        // parents.insert(e2, Parent::new(e1));
        // }
        //
        // frame 3
        // println!("\nFRAME 3:\n---");
        // planner.dispatch(ctx.clone());
        // planner.wait();
    }

    fn construct(n: usize) -> (Planner<Arc<Mutex<Context>>>, Arc<Mutex<Context>>) {
        let mut world = World::new();

        world.register::<LocalTransform>();
        world.register::<Transform>();
        world.register::<Init>();
        world.register::<Parent>();

        for _ in 0..n {
            let transform = LocalTransform::default();

            world.create_now()
                .with::<LocalTransform>(transform)
                .with::<Transform>(Transform::default())
                .build();
        }

        let mut planner: Planner<Arc<Mutex<Context>>> = Planner::new(world, 1);
        let transform_processor = TransformProcessor::new();
        planner.add_system::<TransformProcessor>(transform_processor, "transform_processor", 0);

        let config = Config::default();
        let ctx = Arc::new(Mutex::new(Context::new(config.context_config)));

        (planner, ctx)
    }

    macro_rules! bench_list {
        ($name:ident = $n:expr => $split:expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                let (mut planner, ctx) = construct($n);

                planner.dispatch(ctx.clone());
                planner.wait();

                let mut i = 0;
                planner.run1w0r(move |local: &mut LocalTransform| {
                    if i % $split == 0 {
                        local.dirty.store(true, Ordering::SeqCst);
                        assert!(local.is_dirty());
                    }
                    i += 1;
                });
                planner.wait();

                b.iter(|| {
                    planner.dispatch(ctx.clone());
                    planner.wait();
                });
            }
        }
    }

    // bench_list!(bench_1000_flagged = 1000 => 1);
    // bench_list!(bench_1000_half_flagged = 1000 => 2);
    // bench_list!(bench_1000_third_flagged = 1000 => 3);
    // bench_list!(bench_1000_unflagged = 1000 => u32::max_value());
    //
    // bench_list!(bench_5000_flagged = 5000 => 1);
    // bench_list!(bench_5000_half_flagged = 5000 => 2);
    // bench_list!(bench_5000_third_flagged = 5000 => 3);
    // bench_list!(bench_5000_unflagged = 5000 => u32::max_value());
    //
    // bench_list!(bench_10000_flagged = 10000 => 1);
    // bench_list!(bench_10000_half_flagged = 10000 => 2);
    // bench_list!(bench_10000_third_flagged = 10000 => 3);
    // bench_list!(bench_10000_unflagged = 10000 => u32::max_value());
    //
    // bench_list!(bench_50000_flagged = 50000 => 1);
    // bench_list!(bench_50000_half_flagged = 50000 => 2);
    // bench_list!(bench_50000_third_flagged = 50000 => 3);
    // bench_list!(bench_50000_unflagged = 50000 => u32::max_value());
}
