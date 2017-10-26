//! Scene graph system and types

use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use hibitset::BitSet;
use specs::{Entities, Entity, Join, System, WriteStorage};
use transform::{LocalTransform, Parent, Transform};

/// Handles updating `Transform` components based on the `LocalTransform`
/// component and parents.
#[derive(Default)]
pub struct TransformSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<Entity>,

    init: BitSet,
    frame_init: BitSet,

    dead: HashSet<Entity>,
    remove_parent: Vec<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        Default::default()
    }

    fn remove(&mut self, index: usize) {
        let entity = self.sorted[index];
        self.sorted.swap_remove(index);
        if let Some(swapped) = self.sorted.get(index) {
            self.indices.insert(*swapped, index);
        }
        self.indices.remove(&entity);
        self.init.remove(index as u32);
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

        {
            for (entity, parent) in (&*entities, parents.open().1).join() {
                if parent.entity == entity {
                    self.remove_parent.push(entity);
                }
            }

            for entity in self.remove_parent.iter() {
                eprintln!("Entity was its own parent: {:?}", entity);
                parents.remove(*entity);
            }

            self.remove_parent.clear();
        }

        {
            // Checks for entities with a modified local transform or a modified parent, but isn't initialized yet.
            let filter = locals.open().0 & parents.open().0 & !&self.init; // has a local, parent, and isn't initialized.
            for (entity, _) in (&*entities, &filter).join() {
                self.indices.insert(entity, self.sorted.len());
                self.sorted.push(entity);
                self.frame_init.add(entity.id());
            }
        }

        {
            let locals_flagged = locals.open().1;

            // Compute transforms without parents.
            for (_entity, local, global, _) in
                (&*entities, locals_flagged, &mut globals, !&parents).join()
            {
                global.0 = local.matrix();
                debug_assert!(
                    global.is_finite(),
                    format!("Entity {:?} had a non-finite `Transform`", _entity)
                );
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
                self.dead.contains(&entity),
            ) {
                (Some(parent), Some(local), false) => {
                    // Make sure this iteration isn't a child before the parent.
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

                    // Kill the entity if the parent is dead.
                    if self.dead.contains(&parent.entity) || !entities.is_alive(parent.entity) {
                        self.remove(index);
                        let _ = entities.delete(entity);
                        self.dead.insert(entity);

                        // Re-try index because swapped with last element.
                        continue;
                    }

                    if local_dirty || parent_dirty || globals.open().1.flagged(parent.entity) {
                        let combined_transform = if let Some(parent_global) =
                            globals.get(parent.entity)
                        {
                            (parent_global.0 * local.matrix()).into()
                        } else {
                            local.matrix()
                        };

                        if let Some(global) = globals.get_mut(entity) {
                            global.0 = combined_transform.into();
                        }
                    }
                }
                (_, _, dead @ _) => {
                    // This entity should not be in the sorted list, so remove it.
                    self.remove(index);

                    if !dead && !entities.is_alive(entity) {
                        self.dead.insert(entity);
                    }

                    // Re-try index because swapped with last element.
                    continue;
                }
            }

            index += 1;
        }

        (&mut locals).open().1.clear_flags();
        (&mut parents).open().1.clear_flags();

        for bit in &self.frame_init {
            self.init.add(bit);
        }
        self.frame_init.clear();
        self.dead.clear();
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Decomposed, Matrix4, Quaternion, Vector3, Zero};
    use shred::RunNow;
    use specs::World;
    use transform::{LocalTransform, Parent, Transform, TransformSystem};
    //use quickcheck::{Arbitrary, Gen};

    // If this works, then all other tests should work.
    #[test]
    fn transform_matrix() {
        let mut transform = LocalTransform::default();
        transform.translation = Vector3::new(5.0, 2.0, -0.5);
        transform.rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        transform.scale = Vector3::new(2.0, 2.0, 2.0);

        let decomposed = Decomposed {
            rot: transform.rotation,
            disp: transform.translation,
            scale: 2.0,
        };

        let matrix = transform.matrix();
        let cg_matrix: Matrix4<f32> = decomposed.into();

        assert_eq!(matrix, cg_matrix);
    }

    #[test]
    fn into_from() {
        let transform = Transform::default();
        let primitive: [[f32; 4]; 4] = transform.into();
        assert_eq!(
            primitive,
            <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(transform.0)
        );

        let transform: Transform = primitive.into();
        assert_eq!(
            primitive,
            <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(transform.0)
        );
    }

    fn transform_world<'a, 'b>() -> (World, TransformSystem) {
        let mut world = World::new();
        world.register::<LocalTransform>();
        world.register::<Transform>();
        world.register::<Parent>();

        (world, TransformSystem::new())
    }

    fn together(transform: Transform, local: LocalTransform) -> [[f32; 4]; 4] {
        (transform.0 * local.matrix()).into()
    }

    // Basic default LocalTransform -> Transform (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut world, mut system) = transform_world();

        let mut transform = LocalTransform::default();
        transform.translation = Vector3::zero();
        transform.rotation = Quaternion::new(1.0, 0.0, 0.0, 0.0);

        let e1 = world
            .create_entity()
            .with(transform)
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);

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
        let (mut world, mut system) = transform_world();

        let mut local = LocalTransform::default();
        local.translation = Vector3::new(5.0, 5.0, 5.0);
        local.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e1 = world
            .create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);

        let transform = world.read::<Transform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = local.matrix().into();
        assert_eq!(a1, a2);
    }

    // Test Parent * LocalTransform -> Transform (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut system) = transform_world();

        let mut local1 = LocalTransform::default();
        local1.translation = Vector3::new(5.0, 5.0, 5.0);
        local1.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e1 = world
            .create_entity()
            .with(local1.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = Vector3::new(5.0, 5.0, 5.0);
        local2.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(Transform::default())
            .with(Parent { entity: e1 })
            .build();

        let mut local3 = LocalTransform::default();
        local3.translation = Vector3::new(5.0, 5.0, 5.0);
        local3.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(Transform::default())
            .with(Parent { entity: e2 })
            .build();

        system.run_now(&mut world.res);

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

        {
            let transform3 = transforms.get(e3).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform3.into();
            let a2: [[f32; 4]; 4] = together(transform2, local3);
            assert_eq!(a1, a2);
        };
    }

    // Test Parent * LocalTransform -> Transform (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut system) = transform_world();

        let mut local3 = LocalTransform::default();
        local3.translation = Vector3::new(5.0, 5.0, 5.0);
        local3.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(Transform::default())
            .build();

        let mut local2 = LocalTransform::default();
        local2.translation = Vector3::new(5.0, 5.0, 5.0);
        local2.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(Transform::default())
            .build();

        let mut local1 = LocalTransform::default();
        local1.translation = Vector3::new(5.0, 5.0, 5.0);
        local1.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e1 = world
            .create_entity()
            .with(local1.clone())
            .with(Transform::default())
            .build();

        {
            let mut parents = world.write::<Parent>();
            parents.insert(e2, Parent { entity: e1 });
            parents.insert(e3, Parent { entity: e2 });
        }

        system.run_now(&mut world.res);

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

        {
            let transform3 = transforms.get(e3).unwrap().clone();
            let a1: [[f32; 4]; 4] = transform3.into();
            let a2: [[f32; 4]; 4] = together(transform2, local3);
            assert_eq!(a1, a2);
        };
    }

    #[test]
    #[should_panic]
    fn nan_transform() {
        let (mut world, mut system) = transform_world();

        let mut local = LocalTransform::default();
        // Release the indeterminate forms!
        local.translation = Vector3::new(0.0 / 0.0, 0.0 / 0.0, 0.0 / 0.0);

        world
            .create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);
    }

    #[test]
    #[should_panic]
    fn is_finite_transform() {
        let (mut world, mut system) = transform_world();

        let mut local = LocalTransform::default();
        // Release the indeterminate forms!
        local.translation = Vector3::new(1.0 / 0.0, 1.0 / 0.0, 1.0 / 0.0);
        world
            .create_entity()
            .with(local.clone())
            .with(Transform::default())
            .build();

        system.run_now(&mut world.res);
    }

    #[test]
    fn entity_is_parent() {
        let (mut world, mut system) = transform_world();

        let e3 = world
            .create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        world.write::<Parent>().insert(e3, Parent { entity: e3 });
        system.run_now(&mut world.res);

        let parents = world.read::<Parent>();
        assert_eq!(parents.get(e3), None)
    }

    #[test]
    fn parent_removed() {
        let (mut world, mut system) = transform_world();

        let e1 = world
            .create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        let e2 = world
            .create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .with(Parent { entity: e1 })
            .build();

        let e3 = world
            .create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();

        let e4 = world
            .create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .with(Parent { entity: e3 })
            .build();

        let e5 = world
            .create_entity()
            .with(LocalTransform::default())
            .with(Transform::default())
            .with(Parent { entity: e4 })
            .build();

        let _ = world.delete_entity(e1);
        system.run_now(&mut world.res);
        world.maintain();

        assert_eq!(world.is_alive(e1), false);
        assert_eq!(world.is_alive(e2), false);

        let _ = world.delete_entity(e3);
        system.run_now(&mut world.res);
        system.run_now(&mut world.res);
        world.maintain();

        assert_eq!(world.is_alive(e3), false);
        assert_eq!(world.is_alive(e4), false);
        assert_eq!(world.is_alive(e5), false);
    }
}
