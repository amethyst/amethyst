//! Scene graph system and types

extern crate cgmath;
extern crate specs;
// extern crate test;

use self::cgmath::Matrix4;
use self::specs::{Join, Entity, RunArg, System};

use components::transform::{LocalTransform, Transform, Child, Init};
use std::collections::{HashMap, HashSet};

/// Transformation system.
///
/// Handles updating `Transform` components based on the `LocalTransform` component and parents.
pub struct TransformSystem {
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

impl TransformSystem {
    pub fn new() -> TransformSystem {
        TransformSystem {
            indices: HashMap::new(),
            sorted: Vec::new(),
            new: Vec::new(),
            dead: HashSet::new(),
            dirty: HashSet::new(),
            swapped: HashSet::new(),
        }
    }
}

impl System<()> for TransformSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
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
    use super::cgmath::{Decomposed, Quaternion, Vector3, Matrix4};
    use components::transform::{LocalTransform, Transform};

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
    // let transform_system = TransformSystem::new();
    // planner.add_system::<TransformSystem>(transform_system, "transform_system", 0);
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
