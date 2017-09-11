//! Scene graph system and types

use cgmath::Matrix4;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use specs::{Entities, Entity, Join, ReadStorage, System, WriteStorage};
use hibitset::{BitSet, BitSetNot};
use transform::{LocalTransform, Transform, Parent};

/// Handles updating `Transform` components based on the `LocalTransform`
/// component and parents.
#[derive(Default)]
pub struct TransformSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<(Entity, Entity)>,
    /// Initialized entities.
    init: BitSet,
    /// Entities that have been removed in current frame.
    dead: BitSet,
    /// Child entities that were dirty.
    dirty: BitSet,
    /// Prevent circular infinite loops with parents.
    swapped: BitSet,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem {
            indices: HashMap::default(),
            sorted: Vec::new(),
            init: BitSet::new(),
            /*
            dead: HashSet::default(),
            dirty: HashSet::default(),
            swapped: HashSet::default(),
            */
            dead: BitSet::new(),
            dirty: BitSet::new(),
            swapped: BitSet::new(),
        }
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, LocalTransform>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Transform>
    );

    fn run(&mut self, (entities, locals, parents, mut globals): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");
        // Checks for entities with a local transform and parent, but no
        // `Init` component.
        for (entity, _, parent, id) in (&*entities, &locals, &parents, &BitSetNot(&self.init.clone())).join() {
            self.indices.insert(entity, self.sorted.len());
            self.sorted.push((entity, parent.entity));
            self.init.add(id);
        }

        // Deletes entities whose parents aren't alive.
        for &(entity, _) in &self.sorted {
            if let Some(parent) = parents.get(entity) {
                if !entities.is_alive(parent.entity) || self.dead.contains(parent.entity.id()) {
                    entities.delete(entity);
                    self.dead.add(entity.id());
                }
            }
        }

        {
            let locals_flagged = locals.open().1;

            // Compute transforms without parents.
            for (entity, local, global, _) in (&*entities, locals_flagged, &mut globals, !&parents).join() {
                self.dirty.add(entity.id());
                global.0 = local.matrix();
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let (entity, parent_entity) = self.sorted[index];
            let local_dirty = locals.open().1.flagged(entity);
            let parent_dirty = parents.open().1.flagged(parent_entity);

            match (
                parents.get(entity),
                locals.get(entity),
                self.dead.contains(entity.id()),
            ) {
                (Some(parent), Some(local), false) => {
                    // Make sure the transform is also dirty if the parent has changed.
                    if parent_dirty && !self.swapped.contains(entity.id()) {
                        if parent.entity != parent_entity {
                            self.sorted[index] = (entity, parent.entity);
                        }

                        let mut swap = None;

                        // If the index is none then the parent is an orphan or dead
                        if let Some(parent_index) = self.indices.get(&parent.entity) {
                            if parent_index > &index {
                                swap = Some(*parent_index);
                            }
                        }

                        if let Some(p) = swap {
                            // Swap the parent and child.
                            self.sorted.swap(p, index);
                            self.indices.insert(parent.entity, index);
                            self.indices.insert(entity, p);
                            self.swapped.add(entity.id());

                            // Swap took place, re-try this index.
                            continue;
                        }
                    }

                    if local_dirty || parent_dirty ||
                        self.dirty.contains(parent.entity.id())
                    {
                        let combined_transform = if let Some(parent_global) =
                            globals.get(parent.entity)
                        {
                            (Matrix4::from(parent_global.0) * Matrix4::from(local.matrix())).into()
                        } else {
                            local.matrix()
                        };

                        if let Some(global) = globals.get_mut(entity) {
                            global.0 = combined_transform;
                        }

                        self.dirty.add(entity.id());
                    }
                }
                _ => {
                    self.sorted.swap_remove(index); // swap with last to prevent shift
                    if let Some(swapped) = self.sorted.get(index) {
                        self.indices.insert(swapped.0, index);

                        // Make sure to check for parent swap next iteration
                        parents.open().1.flag(swapped.0)
                    }
                    self.indices.remove(&entity);
                    self.init.remove(entity.id());

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
    use specs::{World, Dispatcher, DispatcherBuilder};
    use transform::{Parent, LocalTransform, Transform, TransformSystem};

    // If this works, then all other tests should work.
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

    fn transform_world<'a, 'b>() -> (World, Dispatcher<'a, 'b>) {
        let mut world = World::new();
        world.register::<LocalTransform>();
        world.register::<Transform>();
        world.register::<Parent>();

        let dispatcher = DispatcherBuilder::new()
            .add(TransformSystem::new(), "amethyst/transform", &[])
            .build();

        (world, dispatcher)
    }

    // Basic default LocalTransform -> Transform (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut world, mut dispatcher) = transform_world();

        let mut transform = LocalTransform::default();
        transform.translation = [0.0, 0.0, 0.0];
        transform.rotation = [1.0, 0.0, 0.0, 0.0];

        let e1 = world.create_entity()
            .with(transform)
            .with(Transform::default())
            .build();

        dispatcher.dispatch(&mut world.res);
        world.maintain();

        let transform = world.read::<Transform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = Transform::default().into();
        assert_eq!(a1, a2);
    }

    // Basic sanity check for LocalTransform -> Transform, no parent relationships
    //
    // Should just put the value of the LocalTransform matrix into the Transform component.
    #[test]
    fn basic() {
        let (mut world, mut dispatcher) = transform_world();

        let mut local = LocalTransform::default();
        local.translation = [5.0, 5.0, 5.0];
        local.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        dispatcher.dispatch(&mut world.res);
        world.maintain();

        let transform = world.read::<Transform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = local.matrix().into();
        assert_eq!(a1, a2);
    }

    // Test Parent * LocalTransform -> Transform (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut dispatcher) = transform_world();

        let mut local = LocalTransform::default();
        local.translation = [5.0, 5.0, 5.0];
        local.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = [5.0, 5.0, 5.0];
        local2.rotation = [1.0, 0.5, 0.5, 0.0];

        let e2 = world.create_entity()
            .with(local2)
            .with(Transform::default())
            .with(Parent::new(e1))
            .build();

        dispatcher.dispatch(&mut world.res);
        world.maintain();

        let transform = world.read::<Transform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = local.matrix().into();
        assert_eq!(a1, a2);
    }

    // Test Parent * LocalTransform -> Transform (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut dispatcher) = transform_world();
    }
    
    // If the entity has a parent entity, but the entity has died.
    #[test]
    fn parent_nonexistent() {
        let (mut world, mut dispatcher) = transform_world();
    }
}
