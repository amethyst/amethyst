//! Scene graph system and types

use cgmath::Matrix4;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use specs::{Entities, Entity, Join, ReadStorage, System, WriteStorage};

use transform::{Child, Init, LocalTransform, Transform};

/// Handles updating `Transform` components based on the `LocalTransform`
/// component and parents.
#[derive(Default)]
pub struct TransformSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<(Entity, Entity)>,
    /// New entities in the current update.
    new: Vec<Entity>,
    /// Entities that have been removed in current frame.
    dead: HashSet<Entity>,
    /// Child entities that were dirty.
    dirty: HashSet<Entity>,
    /// Prevent circular infinite loops with parents.
    swapped: HashSet<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem {
            indices: HashMap::default(),
            sorted: Vec::new(),
            new: Vec::new(),
            dead: HashSet::default(),
            dirty: HashSet::default(),
            swapped: HashSet::default(),
        }
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, LocalTransform>,
        ReadStorage<'a, Child>,
        WriteStorage<'a, Init>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (entities, locals, children, mut init, mut globals): Self::SystemData) {
        // Clear dirty flags on `Transform` storage, before updates go in
        (&mut globals).open().1.clear_flags();

        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");
        // Checks for entities with a local transform and parent, but no
        // `Init` component.
        for (entity, _, child, _) in (&*entities, &locals, &children, !&init).join() {
            self.indices.insert(entity, self.sorted.len());
            self.sorted.push((entity, child.parent()));
            self.new.push(entity);
        }

        // Deletes entities whose parents aren't alive.
        for &(entity, _) in &self.sorted {
            if let Some(child) = children.get(entity) {
                if !entities.is_alive(child.parent()) || self.dead.contains(&child.parent()) {
                    let _ = entities.delete(entity);
                    self.dead.insert(entity);
                }
            }
        }

        // Adds an `Init` component to the entity.
        for entity in self.new.drain(..) {
            init.insert(entity, Init);
        }

        {
            // Compute transforms without parents.
            for (ent, local, (mut global_entry, global_restrict), _) in
                (&*entities, &locals, &mut globals.restrict(), !&children).join()
            {
                if local.is_dirty() {
                    self.dirty.insert(ent);
                    let global = global_restrict.get_mut_unchecked(&mut global_entry);
                    global.0 = local.matrix();
                    local.flag(false);
                }
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let (entity, parent_entity) = self.sorted[index];

            match (
                children.get(entity),
                locals.get(entity),
                self.dead.contains(&entity),
            ) {
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
                                swap = Some(*parent_index);
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

                    if local.is_dirty() || child.is_dirty() || self.dirty.contains(&child.parent())
                    {
                        let combined_transform = if let Some(parent_global) =
                            globals.get(child.parent())
                        {
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
    use cgmath::{Decomposed, Matrix4, Quaternion, Vector3};
    use transform::{LocalTransform, Transform};

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
        world.register::<Init>();
        world.register::<Child>();

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

    // Test basic LocalTransform -> Transform, no parent relationships
    #[test]
    fn basic() {
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

    // Test Parent * LocalTransform -> Transform (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut dispatcher) = transform_world();
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
