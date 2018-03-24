//! Scene graph system and types

use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use hibitset::BitSet;
use specs::prelude::{Entities, Entity, InsertedFlag, Join, ModifiedFlag, ReadStorage, ReaderId,
                     RemovedFlag, System, WriteStorage};
use transform::{GlobalTransform, Parent, Transform};

/// Handles updating `GlobalTransform` components based on the `Transform`
/// component and parents.
pub struct TransformSystem {
    /// Map of entities to index in sorted vec.
    indices: HashMap<Entity, usize>,
    /// Vec of entities with parents before children. Only contains entities
    /// with parents.
    sorted: Vec<Entity>,

    init: BitSet,
    frame_init: BitSet,

    parent_modified: BitSet,
    parent_removed: BitSet,

    local_modified: BitSet,

    global_modified: BitSet,

    inserted_parent_id: ReaderId<InsertedFlag>,
    modified_parent_id: ReaderId<ModifiedFlag>,
    removed_parent_id: ReaderId<RemovedFlag>,

    inserted_local_id: ReaderId<InsertedFlag>,
    modified_local_id: ReaderId<ModifiedFlag>,

    dead: HashSet<Entity>,
    remove_parent: Vec<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new(
        inserted_parent_id: ReaderId<InsertedFlag>,
        modified_parent_id: ReaderId<ModifiedFlag>,
        removed_parent_id: ReaderId<RemovedFlag>,
        inserted_local_id: ReaderId<InsertedFlag>,
        modified_local_id: ReaderId<ModifiedFlag>,
    ) -> TransformSystem {
        TransformSystem {
            inserted_parent_id,
            modified_parent_id,
            removed_parent_id,
            inserted_local_id,
            modified_local_id,
            indices: HashMap::default(),
            sorted: Vec::default(),
            init: BitSet::default(),
            frame_init: BitSet::default(),
            dead: HashSet::default(),
            remove_parent: Vec::default(),
            parent_modified: BitSet::default(),
            parent_removed: BitSet::default(),
            local_modified: BitSet::default(),
            global_modified: BitSet::default(),
        }
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
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Parent>,
        WriteStorage<'a, GlobalTransform>,
    );
    fn run(&mut self, (entities, locals, mut parents, mut globals): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");

        self.parent_modified.clear();
        self.parent_removed.clear();
        self.local_modified.clear();
        self.global_modified.clear();

        parents.populate_inserted(&mut self.inserted_parent_id, &mut self.parent_modified);
        parents.populate_modified(&mut self.modified_parent_id, &mut self.parent_modified);
        parents.populate_removed(&mut self.removed_parent_id, &mut self.parent_removed);

        locals.populate_inserted(&mut self.inserted_local_id, &mut self.local_modified);
        locals.populate_modified(&mut self.modified_local_id, &mut self.local_modified);

        {
            for (entity, _, parent) in (&*entities, &self.parent_modified, &parents).join() {
                if parent.entity == entity {
                    self.remove_parent.push(entity);
                }
            }

            for entity in self.remove_parent.iter() {
                warn!("Entity was its own parent: {:?}", entity);
                parents.remove(*entity);
            }

            self.remove_parent.clear();
        }

        for entity in &self.sorted {
            if self.parent_removed.contains(entity.id()) {
                self.dead.insert(*entity);
            }
        }

        {
            // Checks for entities with a modified local transform or a modified parent, but isn't initialized yet.
            let filter = &self.local_modified & &self.parent_modified & !&self.init; // has a local, parent, and isn't initialized.
            for (entity, _) in (&*entities, &filter).join() {
                self.indices.insert(entity, self.sorted.len());
                self.sorted.push(entity);
                self.frame_init.add(entity.id());
            }
        }

        {
            // Compute transforms without parents.
            for (entity, _, local, global, _) in (
                &*entities,
                &self.local_modified,
                &locals,
                &mut globals,
                !&parents,
            ).join()
            {
                self.global_modified.add(entity.id());
                global.0 = local.matrix();
                debug_assert!(
                    global.is_finite(),
                    format!("Entity {:?} had a non-finite `Transform`", entity)
                );
            }
        }

        // Compute transforms with parents.
        let mut index = 0;
        while index < self.sorted.len() {
            let entity = self.sorted[index];
            let local_dirty = self.local_modified.contains(entity.id());
            let parent_dirty = self.parent_modified.contains(entity.id());

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

                    if local_dirty || parent_dirty
                        || self.global_modified.contains(parent.entity.id())
                    {
                        let combined_transform =
                            if let Some(parent_global) = globals.get(parent.entity) {
                                (parent_global.0 * local.matrix()).into()
                            } else {
                                local.matrix()
                            };

                        if let Some(global) = globals.get_mut(entity) {
                            self.global_modified.add(entity.id());
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

        for bit in &self.frame_init {
            self.init.add(bit);
        }
        self.frame_init.clear();
        self.dead.clear();
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{Decomposed, Matrix4, One, Quaternion, Vector3, Zero};
    use shred::RunNow;
    use specs::prelude::World;
    use transform::{GlobalTransform, Parent, Transform, TransformSystem};
    //use quickcheck::{Arbitrary, Gen};

    // If this works, then all other tests should work.
    #[test]
    fn transform_matrix() {
        let mut transform = Transform::default();
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
        let transform = GlobalTransform::default();
        let primitive: [[f32; 4]; 4] = transform.into();
        assert_eq!(
            primitive,
            <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(transform.0)
        );

        let transform: GlobalTransform = primitive.into();
        assert_eq!(
            primitive,
            <Matrix4<f32> as Into<[[f32; 4]; 4]>>::into(transform.0)
        );
    }

    fn transform_world<'a, 'b>() -> (World, TransformSystem) {
        let mut world = World::new();
        world.register::<Transform>();
        world.register::<GlobalTransform>();
        world.register::<Parent>();

        let (p_insert, p_modify, p_remove, l_insert, l_modify) = {
            let mut locals = world.write::<Transform>();
            let mut parents = world.write::<Parent>();
            (
                parents.track_inserted(),
                parents.track_modified(),
                parents.track_removed(),
                locals.track_inserted(),
                locals.track_modified(),
            )
        };

        (
            world,
            TransformSystem::new(p_insert, p_modify, p_remove, l_insert, l_modify),
        )
    }

    fn together(transform: GlobalTransform, local: Transform) -> [[f32; 4]; 4] {
        (transform.0 * local.matrix()).into()
    }

    // Basic default Transform -> GlobalTransform (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut world, mut system) = transform_world();

        let mut transform = Transform::default();
        transform.translation = Vector3::zero();
        transform.rotation = Quaternion::one();

        let e1 = world
            .create_entity()
            .with(transform)
            .with(GlobalTransform::default())
            .build();

        system.run_now(&mut world.res);

        let transform = world.read::<GlobalTransform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = GlobalTransform::default().into();
        assert_eq!(a1, a2);
    }

    // Basic sanity check for Transform -> GlobalTransform, no parent relationships
    //
    // Should just put the value of the Transform matrix into the GlobalTransform component.
    #[test]
    fn basic() {
        let (mut world, mut system) = transform_world();

        let mut local = Transform::default();
        local.translation = Vector3::new(5.0, 5.0, 5.0);
        local.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e1 = world
            .create_entity()
            .with(local.clone())
            .with(GlobalTransform::default())
            .build();

        system.run_now(&mut world.res);

        let transform = world.read::<GlobalTransform>().get(e1).unwrap().clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = local.matrix().into();
        assert_eq!(a1, a2);
    }

    // Test Parent * Transform -> GlobalTransform (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut system) = transform_world();

        let mut local1 = Transform::default();
        local1.translation = Vector3::new(5.0, 5.0, 5.0);
        local1.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e1 = world
            .create_entity()
            .with(local1.clone())
            .with(GlobalTransform::default())
            .build();

        let mut local2 = Transform::default();
        local2.translation = Vector3::new(5.0, 5.0, 5.0);
        local2.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(GlobalTransform::default())
            .with(Parent { entity: e1 })
            .build();

        let mut local3 = Transform::default();
        local3.translation = Vector3::new(5.0, 5.0, 5.0);
        local3.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(GlobalTransform::default())
            .with(Parent { entity: e2 })
            .build();

        system.run_now(&mut world.res);

        let transforms = world.read::<GlobalTransform>();

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

    // Test Parent * Transform -> GlobalTransform (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut system) = transform_world();

        let mut local3 = Transform::default();
        local3.translation = Vector3::new(5.0, 5.0, 5.0);
        local3.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(GlobalTransform::default())
            .build();

        let mut local2 = Transform::default();
        local2.translation = Vector3::new(5.0, 5.0, 5.0);
        local2.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(GlobalTransform::default())
            .build();

        let mut local1 = Transform::default();
        local1.translation = Vector3::new(5.0, 5.0, 5.0);
        local1.rotation = Quaternion::new(1.0, 0.5, 0.5, 0.0);

        let e1 = world
            .create_entity()
            .with(local1.clone())
            .with(GlobalTransform::default())
            .build();

        {
            let mut parents = world.write::<Parent>();
            parents.insert(e2, Parent { entity: e1 });
            parents.insert(e3, Parent { entity: e2 });
        }

        system.run_now(&mut world.res);

        let transforms = world.read::<GlobalTransform>();

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

        let mut local = Transform::default();
        // Release the indeterminate forms!
        local.translation = Vector3::new(0.0 / 0.0, 0.0 / 0.0, 0.0 / 0.0);

        world
            .create_entity()
            .with(local.clone())
            .with(GlobalTransform::default())
            .build();

        system.run_now(&mut world.res);
    }

    #[test]
    #[should_panic]
    fn is_finite_transform() {
        let (mut world, mut system) = transform_world();

        let mut local = Transform::default();
        // Release the indeterminate forms!
        local.translation = Vector3::new(1.0 / 0.0, 1.0 / 0.0, 1.0 / 0.0);
        world
            .create_entity()
            .with(local.clone())
            .with(GlobalTransform::default())
            .build();

        system.run_now(&mut world.res);
    }

    #[test]
    fn entity_is_parent() {
        let (mut world, mut system) = transform_world();

        let e3 = world
            .create_entity()
            .with(Transform::default())
            .with(GlobalTransform::default())
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
            .with(Transform::default())
            .with(GlobalTransform::default())
            .build();

        let e2 = world
            .create_entity()
            .with(Transform::default())
            .with(GlobalTransform::default())
            .with(Parent { entity: e1 })
            .build();

        let e3 = world
            .create_entity()
            .with(Transform::default())
            .with(GlobalTransform::default())
            .build();

        let e4 = world
            .create_entity()
            .with(Transform::default())
            .with(GlobalTransform::default())
            .with(Parent { entity: e3 })
            .build();

        let e5 = world
            .create_entity()
            .with(Transform::default())
            .with(GlobalTransform::default())
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
