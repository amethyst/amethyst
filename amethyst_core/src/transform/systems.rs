//! Scene graph system and types

use std::marker::PhantomData;

use crate::ecs::prelude::{
    ComponentEvent, Entities, Join, ReadExpect, ReadStorage, ReaderId, Resources, System,
    WriteStorage,
};
use hibitset::BitSet;

use crate::math::RealField;
use crate::transform::{HierarchyEvent, Parent, ParentHierarchy, Transform};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

/// Handles updating `global_matrix` field from `Transform` components.
pub struct TransformSystem<N> {
    local_modified: BitSet,
    locals_events_id: Option<ReaderId<ComponentEvent>>,
    parent_events_id: Option<ReaderId<HierarchyEvent>>,
    _phantom: PhantomData<N>,
}

impl<N> TransformSystem<N> {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem<N> {
        TransformSystem {
            locals_events_id: None,
            parent_events_id: None,
            local_modified: BitSet::default(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, N: RealField> System<'a> for TransformSystem<N> {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ParentHierarchy>,
        WriteStorage<'a, Transform<N>>,
        ReadStorage<'a, Parent>,
    );
    fn run(&mut self, (entities, hierarchy, mut locals, parents): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");

        self.local_modified.clear();

        locals
            .channel()
            .read(
                self.locals_events_id.as_mut().expect(
                    "`TransformSystem::setup` was not called before `TransformSystem::run`",
                ),
            )
            .for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                    self.local_modified.add(*id);
                }
                ComponentEvent::Removed(_id) => {}
            });

        for event in hierarchy.changed().read(
            self.parent_events_id
                .as_mut()
                .expect("`TransformSystem::setup` was not called before `TransformSystem::run`"),
        ) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    // Sometimes the user may have already deleted the entity.
                    // This is fine, so we'll ignore any errors this may give
                    // since it can only fail due to the entity already being dead.
                    let _ = entities.delete(entity);
                }
                HierarchyEvent::Modified(entity) => {
                    self.local_modified.add(entity.id());
                }
            }
        }

        let mut modified = vec![];
        // Compute transforms without parents.
        for (entity, _, local, _) in
            (&*entities, &self.local_modified, &mut locals, !&parents).join()
        {
            modified.push(entity.id());
            local.global_matrix = local.matrix();
            debug_assert!(
                local.is_finite(),
                format!(
                    "Entity {:?} had a non-finite `Transform` {:?}",
                    entity, local
                )
            );
        }
        modified.into_iter().for_each(|id| {
            self.local_modified.add(id);
        });

        // Compute transforms with parents.
        for entity in hierarchy.all() {
            let self_dirty = self.local_modified.contains(entity.id());
            if let Some(parent) = parents.get(*entity) {
                let parent_dirty = self.local_modified.contains(parent.entity.id());
                if parent_dirty || self_dirty {
                    let combined_transform = {
                        let local = locals.get(*entity);
                        if local.is_none() {
                            continue;
                        }
                        let local = local.unwrap();
                        if let Some(parent_global) = locals.get(parent.entity) {
                            (parent_global.global_matrix * local.matrix())
                        } else {
                            local.matrix()
                        }
                    };
                    self.local_modified.add(entity.id());
                    locals.get_mut(*entity).expect("unreachable: We know this entity has a local because is was just modified.").global_matrix = combined_transform;
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use crate::ecs::prelude::SystemData;
        Self::SystemData::setup(res);
        let mut hierarchy = res.fetch_mut::<ParentHierarchy>();
        let mut locals = WriteStorage::<Transform<f32>>::fetch(res);
        self.parent_events_id = Some(hierarchy.track());
        self.locals_events_id = Some(locals.register_reader());
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecs::prelude::{Builder, World},
        math::{Matrix4, Quaternion, RealField, Unit, Vector3},
    };
    use shred::RunNow;
    use specs_hierarchy::{Hierarchy, HierarchySystem};

    use crate::transform::{Parent, Transform, TransformSystem};

    // If this works, then all other tests should work.
    #[test]
    fn transform_matrix() {
        let mut transform = Transform::default();
        transform.set_translation_xyz(5.0, 2.0, -0.5);
        transform.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)));
        transform.set_scale(Vector3::new(2.0, 2.0, 2.0));

        let combined = Matrix4::new_translation(transform.translation())
            * transform.rotation().to_rotation_matrix().to_homogeneous()
            * Matrix4::new_scaling(2.0);

        assert_eq!(transform.matrix(), combined);
    }

    fn transform_world<'a, 'b>() -> (World, HierarchySystem<Parent>, TransformSystem<f32>) {
        let mut world = World::new();
        let mut hs = HierarchySystem::<Parent>::new();
        let mut ts = TransformSystem::<f32>::new();
        hs.setup(&mut world.res);
        ts.setup(&mut world.res);

        (world, hs, ts)
    }

    fn together<N: RealField>(global_matrix: Matrix4<N>, local_matrix: Matrix4<N>) -> Matrix4<N> {
        global_matrix * local_matrix
    }

    // Basic default Transform's local matrix -> global matrix  (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut world, mut hs, mut system) = transform_world();

        let transform = Transform::<f32>::default();

        let e1 = world.create_entity().with(transform).build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let transform = world
            .read_storage::<Transform<f32>>()
            .get(e1)
            .unwrap()
            .clone();
        // let a1: [[f32; 4]; 4] = transform.global_matrix().into();
        // let a2: [[f32; 4]; 4] = Transform::default().global_matrix().into();
        assert_eq!(
            transform.global_matrix(),
            Transform::default().global_matrix()
        );
    }

    // Basic sanity check for Transform's local matrix -> global matrix, no parent relationships
    //
    // Should just put the value of the Transform's local matrix into the global matrix field.
    #[test]
    fn basic() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local = Transform::<f32>::default();
        local.set_translation_xyz(5.0, 5.0, 5.0);
        local.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world.create_entity().with(local.clone()).build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let transform = world
            .read_storage::<Transform<f32>>()
            .get(e1)
            .unwrap()
            .clone();
        let a1 = transform.global_matrix();
        let a2 = local.matrix();
        assert_eq!(*a1, a2);
    }

    // Test Parent's global matrix * Child's local matrix -> Child's global matrix (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local1 = Transform::<f32>::default();
        local1.set_translation_xyz(5.0, 5.0, 5.0);
        local1.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world.create_entity().with(local1.clone()).build();

        let mut local2 = Transform::<f32>::default();
        local2.set_translation_xyz(5.0, 5.0, 5.0);
        local2.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(Parent { entity: e1 })
            .build();

        let mut local3 = Transform::<f32>::default();
        local3.set_translation_xyz(5.0, 5.0, 5.0);
        local3.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(Parent { entity: e2 })
            .build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let e1_transform = world
            .read_storage::<Transform<f32>>()
            .get(e1)
            .unwrap()
            .clone();
        let a1 = e1_transform.global_matrix();
        let a2 = local1.matrix();
        assert_eq!(*a1, a2);

        let e2_transform = world
            .read_storage::<Transform<f32>>()
            .get(e2)
            .unwrap()
            .clone();
        let a3 = e2_transform.global_matrix();
        let a4 = together(*a1, local2.matrix());
        assert_eq!(*a3, a4);

        let e3_transform = world
            .read_storage::<Transform<f32>>()
            .get(e3)
            .unwrap()
            .clone();
        let a3 = e3_transform.global_matrix();
        let _a4 = together(*a3, local3.matrix());
        // assert_eq!(*a3, a4);
        // let global_matrix1 = {
        //     // First entity (top level parent)
        //     let global_matrix1 = local1.global_matrix();
        //     let a1 = global_matrix1;
        //     let a2 = local1.matrix();
        //     assert_eq!(*a1, a2);
        //     global_matrix1
        // };

        // let global_matrix2 = {
        //     let global_matrix2 = local2.global_matrix();
        //     let a1 = global_matrix2;
        //     let a2 = together(*global_matrix1, local2.matrix());
        //     assert_eq!(*a1, a2);
        //     global_matrix2
        // };

        // {
        //     let global_matrix3 = local3.global_matrix();
        //     let a1 = global_matrix3;
        //     let a2 = together(*global_matrix2, local3.matrix());
        //     assert_eq!(*a1, a2);
        // };
    }

    // Test Parent's global matrix * Child's local matrix -> Child's global matrix
    // (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local3 = Transform::<f32>::default();
        local3.set_translation_xyz(5.0, 5.0, 5.0);
        local3.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e3 = world.create_entity().with(local3.clone()).build();

        let mut local2 = Transform::<f32>::default();
        local2.set_translation_xyz(5.0, 5.0, 5.0);
        local2.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e2 = world.create_entity().with(local2.clone()).build();

        let mut local1 = Transform::<f32>::default();
        local1.set_translation_xyz(5.0, 5.0, 5.0);
        local1.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world.create_entity().with(local1.clone()).build();

        // e1 > e2 > e3
        {
            let mut parents = world.write_storage::<Parent>();
            parents.insert(e2, Parent { entity: e1 }).unwrap();
            parents.insert(e3, Parent { entity: e2 }).unwrap();
        }

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let global_matrix1 = {
            let e1_transform = world
                .read_storage::<Transform<f32>>()
                .get(e1)
                .unwrap()
                .clone();

            // First entity (top level parent)
            let a1 = e1_transform.global_matrix().clone();
            let a2 = local1.matrix();
            assert_eq!(a1, a2);
            a1
        };

        let global_matrix2 = {
            let e2_transform = world
                .read_storage::<Transform<f32>>()
                .get(e2)
                .unwrap()
                .clone();

            let a1 = e2_transform.global_matrix().clone();
            let a2 = together(global_matrix1, local2.matrix());
            assert_eq!(a1, a2);
            a1
        };

        {
            let e3_transform = world
                .read_storage::<Transform<f32>>()
                .get(e3)
                .unwrap()
                .clone();

            let a1 = e3_transform.global_matrix();
            let a2 = together(global_matrix2, local3.matrix());
            assert_eq!(*a1, a2);
        };
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn nan_transform() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local = Transform::<f32>::default();
        // Release the indeterminate forms!
        local.set_translation_xyz(0.0 / 0.0, 0.0 / 0.0, 0.0 / 0.0);

        world.create_entity().with(local.clone()).build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn is_finite_transform() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local = Transform::<f32>::default();
        // Release the indeterminate forms!
        local.set_translation_xyz(1.0 / 0.0, 1.0 / 0.0, 1.0 / 0.0);
        world.create_entity().with(local.clone()).build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
    }

    #[test]
    fn parent_removed() {
        let (mut world, mut hs, mut system) = transform_world();

        let e1 = world
            .create_entity()
            .with(Transform::<f32>::default())
            .build();

        let e2 = world
            .create_entity()
            .with(Transform::<f32>::default())
            .with(Parent { entity: e1 })
            .build();

        let e3 = world
            .create_entity()
            .with(Transform::<f32>::default())
            .build();

        let e4 = world
            .create_entity()
            .with(Transform::<f32>::default())
            .with(Parent { entity: e3 })
            .build();

        let e5 = world
            .create_entity()
            .with(Transform::<f32>::default())
            .with(Parent { entity: e4 })
            .build();
        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
        world.maintain();
        println!("{:?}", world.read_resource::<Hierarchy<Parent>>().all());

        let _ = world.delete_entity(e1);
        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
        world.maintain();
        println!("{:?}", world.read_resource::<Hierarchy<Parent>>().all());

        assert_eq!(world.is_alive(e1), false);
        assert_eq!(world.is_alive(e2), false);

        let _ = world.delete_entity(e3);
        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
        world.maintain();
        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
        world.maintain();

        assert_eq!(world.is_alive(e3), false);
        assert_eq!(world.is_alive(e4), false);
        assert_eq!(world.is_alive(e5), false);
    }
}
