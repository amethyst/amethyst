//! Scene graph system and types

use hibitset::BitSet;
use specs::prelude::{
    ComponentEvent, Entities, Entity, Join, ReadExpect, ReadStorage, ReaderId, Resources, System,
    WriteStorage,
};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::transform::{GlobalTransform, HierarchyEvent, Parent, ParentHierarchy, Transform};

/// Handles updating `GlobalTransform` components based on the `Transform`
/// component and parents.
pub struct TransformSystem {
    local_modified: BitSet,
    global_modified: BitSet,

    locals_events_id: Option<ReaderId<ComponentEvent>>,

    parent_events_id: Option<ReaderId<HierarchyEvent>>,

    scratch: Vec<Entity>,
}

impl TransformSystem {
    /// Creates a new transform processor.
    pub fn new() -> TransformSystem {
        TransformSystem {
            locals_events_id: None,
            parent_events_id: None,
            local_modified: BitSet::default(),
            global_modified: BitSet::default(),
            scratch: Vec::new(),
        }
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ParentHierarchy>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Parent>,
        WriteStorage<'a, GlobalTransform>,
    );
    fn run(&mut self, (entities, hierarchy, locals, parents, mut globals): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("transform_system");

        self.scratch.clear();
        self.scratch
            .extend((&*entities, &locals, !&globals).join().map(|d| d.0));
        for entity in &self.scratch {
            globals
                .insert(*entity, GlobalTransform::default())
                .expect("unreachable");
        }

        self.local_modified.clear();
        self.global_modified.clear();

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

        // Compute transforms without parents.
        for (entity, _, local, global, _) in (
            &*entities,
            &self.local_modified,
            &locals,
            &mut globals,
            !&parents,
        )
            .join()
        {
            self.global_modified.add(entity.id());
            global.0 = local.matrix();
            debug_assert!(
                global.is_finite(),
                format!("Entity {:?} had a non-finite `Transform`", entity)
            );
        }

        // Compute transforms with parents.
        for entity in hierarchy.all() {
            let self_dirty = self.local_modified.contains(entity.id());
            if let (Some(parent), Some(local)) = (parents.get(*entity), locals.get(*entity)) {
                let parent_dirty = self.global_modified.contains(parent.entity.id());
                if parent_dirty || self_dirty {
                    let combined_transform = if let Some(parent_global) = globals.get(parent.entity)
                    {
                        (parent_global.0 * local.matrix())
                    } else {
                        local.matrix()
                    };

                    if let Some(global) = globals.get_mut(*entity) {
                        self.global_modified.add(entity.id());
                        global.0 = combined_transform;
                    }
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        use specs::prelude::SystemData;
        Self::SystemData::setup(res);
        let mut hierarchy = res.fetch_mut::<ParentHierarchy>();
        let mut locals = WriteStorage::<Transform>::fetch(res);
        self.parent_events_id = Some(hierarchy.track());
        self.locals_events_id = Some(locals.register_reader());
    }
}

#[cfg(test)]
mod tests {
    use nalgebra::{Matrix4, Quaternion, Unit};
    use shred::RunNow;
    use specs::prelude::{Builder, World};
    use specs_hierarchy::{Hierarchy, HierarchySystem};

    use crate::transform::{GlobalTransform, Parent, Transform, TransformSystem};

    // If this works, then all other tests should work.
    #[test]
    fn transform_matrix() {
        let mut transform = Transform::default();
        transform.set_xyz(5.0, 2.0, -0.5);
        transform.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)));
        transform.set_scale(2.0, 2.0, 2.0);

        let combined = Matrix4::new_translation(transform.translation())
            * transform.rotation().to_rotation_matrix().to_homogeneous()
            * Matrix4::new_scaling(2.0);

        assert_eq!(transform.matrix(), combined);
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

    fn transform_world<'a, 'b>() -> (World, HierarchySystem<Parent>, TransformSystem) {
        let mut world = World::new();
        let mut hs = HierarchySystem::<Parent>::new();
        let mut ts = TransformSystem::new();
        hs.setup(&mut world.res);
        ts.setup(&mut world.res);

        (world, hs, ts)
    }

    fn together(transform: GlobalTransform, local: Transform) -> [[f32; 4]; 4] {
        (transform.0 * local.matrix()).into()
    }

    // Basic default Transform -> GlobalTransform (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut world, mut hs, mut system) = transform_world();

        let transform = Transform::default();

        let e1 = world
            .create_entity()
            .with(transform)
            .with(GlobalTransform::default())
            .build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let transform = world
            .read_storage::<GlobalTransform>()
            .get(e1)
            .unwrap()
            .clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = GlobalTransform::default().into();
        assert_eq!(a1, a2);
    }

    // Basic sanity check for Transform -> GlobalTransform, no parent relationships
    //
    // Should just put the value of the Transform matrix into the GlobalTransform component.
    #[test]
    fn basic() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local = Transform::default();
        local.set_xyz(5.0, 5.0, 5.0);
        local.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world
            .create_entity()
            .with(local.clone())
            .with(GlobalTransform::default())
            .build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let transform = world
            .read_storage::<GlobalTransform>()
            .get(e1)
            .unwrap()
            .clone();
        let a1: [[f32; 4]; 4] = transform.into();
        let a2: [[f32; 4]; 4] = local.matrix().into();
        assert_eq!(a1, a2);
    }

    // Test Parent * Transform -> GlobalTransform (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local1 = Transform::default();
        local1.set_xyz(5.0, 5.0, 5.0);
        local1.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world
            .create_entity()
            .with(local1.clone())
            .with(GlobalTransform::default())
            .build();

        let mut local2 = Transform::default();
        local2.set_xyz(5.0, 5.0, 5.0);
        local2.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(GlobalTransform::default())
            .with(Parent { entity: e1 })
            .build();

        let mut local3 = Transform::default();
        local3.set_xyz(5.0, 5.0, 5.0);
        local3.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(GlobalTransform::default())
            .with(Parent { entity: e2 })
            .build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let transforms = world.read_storage::<GlobalTransform>();

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

    // Test Parent * Transform -> GlobalTransform
    // (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local3 = Transform::default();
        local3.set_xyz(5.0, 5.0, 5.0);
        local3.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e3 = world
            .create_entity()
            .with(local3.clone())
            .with(GlobalTransform::default())
            .build();

        let mut local2 = Transform::default();
        local2.set_xyz(5.0, 5.0, 5.0);
        local2.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e2 = world
            .create_entity()
            .with(local2.clone())
            .with(GlobalTransform::default())
            .build();

        let mut local1 = Transform::default();
        local1.set_xyz(5.0, 5.0, 5.0);
        local1.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world
            .create_entity()
            .with(local1.clone())
            .with(GlobalTransform::default())
            .build();

        {
            let mut parents = world.write_storage::<Parent>();
            parents.insert(e2, Parent { entity: e1 }).unwrap();
            parents.insert(e3, Parent { entity: e2 }).unwrap();
        }

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);

        let transforms = world.read_storage::<GlobalTransform>();

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
    #[cfg(debug_assertions)]
    fn nan_transform() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local = Transform::default();
        // Release the indeterminate forms!
        local.set_xyz(0.0 / 0.0, 0.0 / 0.0, 0.0 / 0.0);

        world
            .create_entity()
            .with(local.clone())
            .with(GlobalTransform::default())
            .build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn is_finite_transform() {
        let (mut world, mut hs, mut system) = transform_world();

        let mut local = Transform::default();
        // Release the indeterminate forms!
        local.set_xyz(1.0 / 0.0, 1.0 / 0.0, 1.0 / 0.0);
        world
            .create_entity()
            .with(local.clone())
            .with(GlobalTransform::default())
            .build();

        hs.run_now(&mut world.res);
        system.run_now(&mut world.res);
    }

    #[test]
    fn parent_removed() {
        let (mut world, mut hs, mut system) = transform_world();

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
