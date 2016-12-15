//! Scene graph processor and types

extern crate cgmath;
// extern crate test;

use self::cgmath::{Quaternion, Vector3, Matrix3, Matrix4};

use ecs::{Join, Component, NullStorage, VecStorage, Entity, RunArg, Processor};
use context::Context;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct InnerTransform {
    /// Translation/position vector [x, y, z]
    pub translation: [f32; 3],

    /// Quaternion [w (scalar), x, y, z]
    pub rotation: [f32; 4],

    /// Scale vector [x, y, z]
    pub scale: [f32; 3],
}

/// Local position, rotation, and scale (from parent if it exists).
#[derive(Debug)]
pub struct LocalTransform {
    /// Wrapper around the transform data for dirty flag setting.
    wrapped: InnerTransform,

    /// Flag for re-computation
    dirty: AtomicBool,
}

impl Deref for LocalTransform {
    type Target = InnerTransform;
    fn deref(&self) -> &InnerTransform {
        &self.wrapped
    }
}

impl DerefMut for LocalTransform {
    fn deref_mut(&mut self) -> &mut InnerTransform {
        self.flag(true);
        &mut self.wrapped
    }
}

impl LocalTransform {
    /// Flags the current transform for re-computation.
    ///
    /// Note: All `set_*` methods will automatically flag the component.
    #[inline]
    pub fn flag(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst);
    }

    /// Returns whether or not the current transform is flagged for re-computation or "dirty".
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    /// Returns the local object matrix for the transform.
    ///
    /// Combined with the parent's global `Transform` component it gives
    /// the global (or world) matrix for the current entity.
    #[inline]
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let quat: Matrix3<f32> = Quaternion::from(self.rotation).into();
        let scale: Matrix3<f32> = Matrix3::<f32> {
            x: [self.scale[0], 0.0, 0.0].into(),
            y: [0.0, self.scale[1], 0.0].into(),
            z: [0.0, 0.0, self.scale[2]].into(),
        };
        let mut matrix: Matrix4<f32> = (&quat * scale).into();
        matrix.w = Vector3::from(self.translation).extend(1.0f32);
        matrix.into()
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        LocalTransform {
            wrapped: InnerTransform {
                translation: [0.0, 0.0, 0.0],
                rotation: [1.0, 0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            },
            dirty: AtomicBool::new(true),
        }
    }
}

impl Component for LocalTransform {
    type Storage = VecStorage<LocalTransform>;
}

/// Absolute transformation (transformed from origin).
/// Used for rendering position and orientation.
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

impl From<[[f32; 4]; 4]> for Transform {
    fn from(matrix: [[f32; 4]; 4]) -> Self {
        Transform(matrix)
    }
}

impl Into<[[f32; 4]; 4]> for Transform {
    fn into(self) -> [[f32; 4]; 4] {
        self.0
    }
}

/// Initialization flag.
/// Added to entity with a `LocalTransform` component after the first update.
#[derive(Default, Copy, Clone)]
pub struct Init;
impl Component for Init {
    type Storage = NullStorage<Init>;
}

/// Component for defining a parent entity.
pub struct Child {
    /// The parent entity
    parent: Entity,

    /// Flag for whether the child was changed
    dirty: AtomicBool,
}

impl Child {
    pub fn new(entity: Entity) -> Child {
        Child {
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
        self.flag(true);
    }

    /// Flag that parent has been changed
    ///
    /// Note: `set_parent` flags the parent.
    #[inline]
    pub fn flag(&self, dirty: bool) {
        self.dirty.store(dirty, Ordering::SeqCst);
    }

    /// Returns whether the parent was changed.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }
}

impl Component for Child {
    type Storage = VecStorage<Child>;
}

/// Transformation processor.
///
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

    // Child entities that were dirty.
    dirty: HashSet<Entity>,

    // Prevent circular infinite loops with parents.
    swapped: HashSet<Entity>,
}

impl TransformProcessor {
    pub fn new() -> TransformProcessor {
        TransformProcessor {
            indices: HashMap::new(),
            sorted: Vec::new(),
            new: Vec::new(),
            dead: HashSet::new(),
            dirty: HashSet::new(),
            swapped: HashSet::new(),
        }
    }
}

impl Processor<Arc<Mutex<Context>>> for TransformProcessor {
    fn run(&mut self, arg: RunArg, _: Arc<Mutex<Context>>) {
        // Fetch world and gets entities/components
        let (entities, locals, mut globals, mut init, children) = arg.fetch(|w| {
            let entities = w.entities();
            let locals = w.read::<LocalTransform>();
            let children = w.read::<Child>();
            let init = w.write::<Init>();

            // Checks for entities with a local transform and parent, but no `Init` component.
            for (entity, _, child, _) in (&entities, &locals, &children, !&init).iter() {
                self.indices.insert(entity, self.sorted.len());
                self.sorted.push((entity, child.parent()));
                self.new.push(entity.clone());
            }

            // Deletes entities whose parents aren't alive.
            for &(entity, _) in &self.sorted {
                if let Some(child) = children.get(entity) {
                    if !w.is_alive(child.parent()) || self.dead.contains(&child.parent()) {
                        arg.delete(entity);
                        self.dead.insert(entity);
                    }
                }
            }

            (entities, locals, w.write::<Transform>(), init, children)
        });

        // Adds an `Init` component to the entity.
        for entity in self.new.drain(..) {
            init.insert(entity, Init);
        }

        {
            let without_parents = (
                &entities,
                &locals,
                &mut globals,
                !&children,
            ).iter();

            // Compute transforms without parents.
            for (ent, local, global, _) in without_parents {
                if local.is_dirty() {
                    self.dirty.insert(ent);
                    global.0 = local.matrix();
                    local.flag(false);
                }
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let (entity, parent_entity) = self.sorted[index];

            match (children.get(entity), locals.get(entity), self.dead.contains(&entity)) {
                (Some(child), Some(local), false) => {
                    // Make sure the transform is also dirty if the parent has changed.
                    if child.is_dirty() && !self.swapped.contains(&entity) {
                        if child.parent() != parent_entity {
                            self.sorted[index] = (entity, child.parent());
                        }

                        let mut swap = None;

                        // If the index is none then the parent is an orphan or dead
                        if let Some(parent_index) = self.indices.get(&child.parent()) {
                            if parent_index > &index {
                                swap = Some(parent_index.clone());
                            }
                        }

                        if let Some(p) = swap {
                            // Swap the parent and child.
                            self.sorted.swap(p, index);
                            self.indices.insert(child.parent(), index);
                            self.indices.insert(entity, p);
                            self.swapped.insert(entity);

                            // Swap took place, re-try this index.
                            continue;
                        }
                    }

                    if local.is_dirty() || child.is_dirty() ||
                       self.dirty.contains(&child.parent()) {
                        let combined_transform = if let Some(parent_global) =
                                                        globals.get(child.parent()) {
                            (Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())).into()
                        } else {
                            local.matrix()
                        };

                        if let Some(global) = globals.get_mut(entity) {
                            global.0 = combined_transform;
                        }

                        local.flag(false);
                        child.flag(false);
                        self.dirty.insert(entity);
                    }
                }
                _ => {
                    self.sorted.swap_remove(index); // swap with last to prevent shift
                    if let Some(swapped) = self.sorted.get(index) {
                        self.indices.insert(swapped.0, index);

                        // Make sure to check for parent swap next iteration
                        if let Some(parent) = children.get(swapped.0) {
                            parent.flag(true);
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
        self.dead.clear();
        self.swapped.clear();
    }
}

#[cfg(test)]
mod tests {
    // use super::test::Bencher;
    use super::*;
    use super::cgmath::{Decomposed, Quaternion, Vector3, Matrix4};

    #[test]
    fn transform_matrix() {
        let mut transform = LocalTransform::default();
        transform.translation = [5.0, 2.0, -0.5];
        transform.rotation = [1.0, 0.0, 0.0, 0.0];
        transform.scale = [2.0, 2.0, 2.0];

        let decomposed = Decomposed {
            rot: Quaternion::from(transform.rotation),
            disp: Vector3::from(transform.translation),
            scale: 2.0,
        };

        let matrix = transform.matrix();
        let cg_matrix: Matrix4<f32> = decomposed.into();
        let cg_matrix: [[f32; 4]; 4] = cg_matrix.into();

        assert_eq!(matrix, cg_matrix);
    }

    #[test]
    fn into_from() {
        let transform = Transform::default();
        let primitive: [[f32; 4]; 4] = transform.into();
        assert_eq!(primitive, transform.0);

        let transform: Transform = primitive.into();
        assert_eq!(primitive, transform.0);
    }

    // fn construct(n: usize) -> (Planner<Arc<Mutex<Context>>>, Arc<Mutex<Context>>) {
    // let mut world = World::new();
    //
    // world.register::<LocalTransform>();
    // world.register::<Transform>();
    // world.register::<Init>();
    // world.register::<Child>();
    //
    // for _ in 0..n {
    // let transform = LocalTransform::default();
    //
    // world.create_now()
    // .with::<LocalTransform>(transform)
    // .with::<Transform>(Transform::default())
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
    // (planner, ctx)
    // }
    //
    // macro_rules! bench_list {
    // ($name:ident = $n:expr => $split:expr) => {
    // #[bench]
    // fn $name(b: &mut Bencher) {
    // let (mut planner, ctx) = construct($n);
    //
    // planner.dispatch(ctx.clone());
    // planner.wait();
    //
    // let mut i = 0;
    // planner.run1w0r(move |local: &mut LocalTransform| {
    // if i % $split == 0 {
    // local.dirty.store(true, Ordering::SeqCst);
    // assert!(local.is_dirty());
    // }
    // i += 1;
    // });
    // planner.wait();
    //
    // b.iter(|| {
    // planner.dispatch(ctx.clone());
    // planner.wait();
    // });
    // }
    // }
    // }

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
