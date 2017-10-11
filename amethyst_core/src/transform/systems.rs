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
    sorted: Vec<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem {
            indices: HashMap::default(),
            sorted: Vec::new(),
        }
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, LocalTransform>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, Transform>,
    );
    fn run(&mut self, (entities, mut locals, mut parents, mut globals): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");

        // Clear dirty flags on `Transform` storage, before updates go in
        (&mut globals).open().1.clear_flags();

        // Checks for entities with a local transform and parent, but isn't initialized yet.
        for (entity, _, _) in (&*entities, &locals, &parents).join() {
            let mut initialized = false;
            if let Some(_) = self.indices.get(&entity) {
                initialized = true;
            }

            if !initialized {
                self.indices.insert(entity, self.sorted.len());
                self.sorted.push(entity);
            }
        }


        {
            let locals_flagged = locals.open().1;

            // Compute transforms without parents.
            for (entity, local, global, _) in (&*entities, locals_flagged, &mut globals, !&parents).join() {
                global.0 = local.matrix();
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let entity = self.sorted[index];
            let local_dirty = locals.open().1.flagged(entity);
            let parent_dirty = parents.open().1.flagged(entity);

            match (
                parents.get(entity),
                locals.get(entity),
            ) {
                (Some(parent), Some(local)) => {
                    // Make sure the transform is also dirty if the parent has changed.
                    if parent_dirty {
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

                            // Swap took place, re-try this index.
                            continue;
                        }
                    }

                    if local_dirty || parent_dirty || globals.open().1.flagged(parent.entity) {
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
                    }
                }
                _ => { // This entity should not be in the sorted list, so remove it.
                    self.sorted.swap_remove(index);
                    if let Some(swapped) = self.sorted.get(index) {
                        self.indices.insert(*swapped, index);
                    }
                    self.indices.remove(&entity);

                    // Re-try index because swapped with last element.
                    continue;
                }
            }

            index += 1;
        }

        (&mut locals).open().1.clear_flags();
        (&mut parents).open().1.clear_flags();
    }
}

#[cfg(test)]
mod tests {
    use specs::{World, Dispatcher, DispatcherBuilder};
    use transform::{Parent, LocalTransform, Transform, TransformSystem};
    use cgmath::{Decomposed, Quaternion, Vector3, Matrix4};

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

    fn together(transform: Transform, local: LocalTransform) -> [[f32; 4]; 4] {
        (Matrix4::from(transform.0) * Matrix4::from(local.matrix())).into()
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

        let mut local1 = LocalTransform::default();
        local1.translation = [5.0, 5.0, 5.0];
        local1.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local1.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = [5.0, 5.0, 5.0];
        local2.rotation = [1.0, 0.5, 0.5, 0.0];

        let e2 = world.create_entity()
            .with(local2.clone())
            .with(Transform::default())
            .with(Parent { entity: e1 })
            .build();

        let mut local3 = LocalTransform::default();
        local3.translation = [5.0, 5.0, 5.0];
        local3.rotation = [1.0, 0.5, 0.5, 0.0];

        let e3 = world.create_entity()
            .with(local3.clone())
            .with(Transform::default())
            .with(Parent { entity: e2 })
            .build();

        dispatcher.dispatch(&mut world.res);
        world.maintain();

        let transforms = world.read::<Transform>();

        let transform1 = {
            // First entity (top level parent)
            let transform1 = transforms.get(e1).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform1.into();
            let a2: [[f32; 4]; 4] = local1.matrix().into();
            assert_eq!(a1, a2);
            transform1
        };

        let transform2 = {
            let transform2 = transforms.get(e2).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform2.into();
            let a2: [[f32; 4]; 4] = together(transform1, local2);
            assert_eq!(a1, a2);
            transform2
        };

        let transform3 = {
            let transform3 = transforms.get(e3).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform3.into();
            let a2: [[f32; 4]; 4] = together(transform2, local3);
            assert_eq!(a1, a2);
            transform3
        };
    }

    // Test Parent * LocalTransform -> Transform (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut dispatcher) = transform_world();

        let mut local3 = LocalTransform::default();
        local3.translation = [5.0, 5.0, 5.0];
        local3.rotation = [1.0, 0.5, 0.5, 0.0];

        let e3 = world.create_entity()
            .with(local3.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = [5.0, 5.0, 5.0];
        local2.rotation = [1.0, 0.5, 0.5, 0.0];

        let e2 = world.create_entity()
            .with(local2.clone())
            .with(Transform::default())
            .build();

        let mut local1 = LocalTransform::default();
        local1.translation = [5.0, 5.0, 5.0];
        local1.rotation = [1.0, 0.5, 0.5, 0.0];

        let e1 = world.create_entity()
            .with(local1.clone())
            .with(Transform::default())
            .build();

        {
            let mut parents = world.write::<Parent>();
            parents.insert(e2, Parent { entity: e1 }); 
            parents.insert(e3, Parent { entity: e2 }); 
        }

        dispatcher.dispatch(&mut world.res);
        world.maintain();
        dispatcher.dispatch(&mut world.res);
        world.maintain();

        let transforms = world.read::<Transform>();

        let transform1 = {
            // First entity (top level parent)
            let transform1 = transforms.get(e1).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform1.into();
            let a2: [[f32; 4]; 4] = local1.matrix().into();
            assert_eq!(a1, a2);
            transform1
        };

        let transform2 = {
            let transform2 = transforms.get(e2).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform2.into();
            let a2: [[f32; 4]; 4] = together(transform1, local2);
            assert_eq!(a1, a2);
            transform2
        };

        let transform3 = {
            let transform3 = transforms.get(e3).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform3.into();
            let a2: [[f32; 4]; 4] = together(transform2, local3);
            assert_eq!(a1, a2);
            transform3
        };
    }
    
    // If the entity has a parent entity, but the entity has died.
    #[test]
    fn parent_nonexistent() {
        let (mut world, mut dispatcher) = transform_world();
    }
}
