//! System that updates global transform matrices based on hierarchy relations.

use super::components::*;
use crate::ecs::*;

/// System that updates global transform matrices based on hierarchy relations.
#[derive(Debug)]
pub struct TransformSystem;

impl System for TransformSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("TransformSystem")
                // Entities at the hierarchy root (no parent component)
                .with_query(
                    <(Entity, &mut Transform)>::query()
                        .filter(maybe_changed::<Transform>() & !component::<Parent>()),
                )
                // Entities that are children of some entity
                .with_query(
                    <(Entity, &mut Transform)>::query()
                        .filter(maybe_changed::<Transform>() & component::<Parent>()),
                )
                .with_query(<(Entity, &Parent)>::query())
                .write_component::<Transform>()
                .build(
                    move |_commands,
                          world,
                          _resource,
                          (query_root, query_children, query_parent)| {
                        // Update global transform for entities that are root of the hierarchy
                        for (entity, transform) in query_root.iter_mut(world) {
                            transform.global_matrix = transform.matrix();
                            debug_assert!(
                                transform.is_finite(),
                                    "Entity {:?} had a non-finite `Transform` {:?}",
                                    entity, transform
                            );
                        }

                        // Update parent transforms for entities in the hierarchy
                        // Ideally we would check if the linked parent entity exists before
                        // accessing it's Transform component, but as the parent_update_system panics
                        // in both cases it is ok to just unwrap/expect here.
                        let (left, mut right) = world.split_for_query(query_parent);
                        for (entity, parent) in query_parent.iter(&left) {
                            let parent_has_transform = right
                                .entry_ref(parent.0)
                                .expect("Invalid entity in Parent component")
                                .into_component::<Transform>()
                                .is_ok();

                            if parent_has_transform {
                                let parent_matrix = right
                                    .entry_ref(parent.0)
                                    .expect("Invalid entity in Parent component")
                                    .into_component::<Transform>()
                                    .unwrap()
                                    .global_matrix;

                                if let Some(transform) = right
                                    .entry_mut(*entity)
                                    .ok()
                                    .and_then(|entry| entry.into_component_mut::<Transform>().ok())
                                {
                                    transform.parent_matrix = parent_matrix;
                                }
                            }
                        }

                        // Update global transform for entities that are children of some entity
                        for (entity, transform) in query_children.iter_mut(world) {
                            transform.global_matrix = transform.parent_matrix * transform.matrix();
                            debug_assert!(
                                transform.is_finite(),
                                    "Entity {:?} had a non-finite `Transform` {:?}",
                                    entity, transform
                            );
                        }
                    },
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecs::*,
        math::{Matrix4, Quaternion, Unit, Vector3},
        transform::{Parent, Transform, TransformBundle},
    };

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

    fn transform_world() -> (Resources, World, Dispatcher) {
        let mut resources = Resources::default();
        let mut world = World::default();

        let dispatcher = DispatcherBuilder::default()
            .add_bundle(TransformBundle)
            .build(&mut world, &mut resources)
            .unwrap();

        (resources, world, dispatcher)
    }

    fn together(global_matrix: Matrix4<f32>, local_matrix: Matrix4<f32>) -> Matrix4<f32> {
        global_matrix * local_matrix
    }

    // Basic default Transform's local matrix -> global matrix  (Should just be identity)
    #[test]
    fn zeroed() {
        let (mut res, mut world, mut dispatcher) = transform_world();

        let transform = Transform::default();

        let e1 = world.push((transform,));

        dispatcher.execute(&mut world, &mut res);

        let transform = world
            .entry(e1)
            .unwrap()
            .into_component::<Transform>()
            .unwrap();

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
        let (mut res, mut world, mut dispatcher) = transform_world();

        let mut local = Transform::default();
        local.set_translation_xyz(5.0, 5.0, 5.0);
        local.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world.push((local,));

        dispatcher.execute(&mut world, &mut res);

        let transform = world
            .entry(e1)
            .unwrap()
            .into_component::<Transform>()
            .unwrap();
        let a1 = transform.global_matrix();
        let a2 = local.matrix();
        assert_eq!(*a1, a2);
    }

    // Test Parent's global matrix * Child's local matrix -> Child's global matrix (Parent is before child)
    #[test]
    fn parent_before() {
        let (mut res, mut world, mut dispatcher) = transform_world();

        let mut local1 = Transform::default();
        local1.set_translation_xyz(1.0, 5.0, 5.0);
        local1.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e1 = world.push((local1,));

        let mut local2 = Transform::default();
        local2.set_translation_xyz(5.0, 1.0, 5.0);
        local2.set_rotation(Unit::new_normalize(Quaternion::new(0.5, 1.0, 0.5, 0.0)));

        let e2 = world.push((local2, Parent(e1)));

        let mut local3 = Transform::default();
        local3.set_translation_xyz(5.0, 5.0, 1.0);
        local3.set_rotation(Unit::new_normalize(Quaternion::new(0.5, 0.5, 1.0, 0.0)));

        let e3 = world.push((local3, Parent(e2)));

        // Current transform system is inneficient and needs multiple passed for deep hierarchy
        dispatcher.execute(&mut world, &mut res);
        dispatcher.execute(&mut world, &mut res);

        let e1_transform = *world
            .entry(e1)
            .unwrap()
            .into_component::<Transform>()
            .unwrap();
        let a1 = e1_transform.global_matrix();
        let a2 = local1.matrix();
        assert_eq!(*a1, a2);

        let e2_transform = *world
            .entry(e2)
            .unwrap()
            .into_component::<Transform>()
            .unwrap();
        let a3 = e2_transform.global_matrix();
        let a4 = together(*a1, local2.matrix());
        assert_eq!(*a3, a4);

        let e3_transform = *world
            .entry(e3)
            .unwrap()
            .into_component::<Transform>()
            .unwrap();
        let a5 = e3_transform.global_matrix();
        let a6 = together(*a3, local3.matrix());
        assert_eq!(*a5, a6);
    }

    // Test Parent's global matrix * Child's local matrix -> Child's global matrix
    // (Parent is after child, therefore must be special cased in list)
    #[test]
    fn parent_after() {
        let (mut res, mut world, mut dispatcher) = transform_world();

        let mut local3 = Transform::default();
        local3.set_translation_xyz(1.0, 5.0, 5.0);
        local3.set_rotation(Unit::new_normalize(Quaternion::new(1.0, 0.5, 0.5, 0.0)));

        let e3 = world.push((local3,));

        let mut local2 = Transform::default();
        local2.set_translation_xyz(5.0, 1.0, 5.0);
        local2.set_rotation(Unit::new_normalize(Quaternion::new(0.5, 1.0, 0.5, 0.0)));

        let e2 = world.push((local2,));

        let mut local1 = Transform::default();
        local1.set_translation_xyz(5.0, 5.0, 1.0);
        local1.set_rotation(Unit::new_normalize(Quaternion::new(0.5, 0.5, 1.0, 0.0)));

        let e1 = world.push((local1,));

        // e1 > e2 > e3
        world.entry(e2).unwrap().add_component(Parent(e1));
        world.entry(e3).unwrap().add_component(Parent(e2));

        // Current transform system is inneficient and needs multiple passed for deep hierarchy
        dispatcher.execute(&mut world, &mut res);
        dispatcher.execute(&mut world, &mut res);

        let global_matrix1 = {
            let e1_transform = world
                .entry(e1)
                .unwrap()
                .into_component::<Transform>()
                .unwrap();

            // First entity (top level parent)
            let a1 = *e1_transform.global_matrix();
            let a2 = local1.matrix();
            assert_eq!(a1, a2);
            a1
        };

        let global_matrix2 = {
            let e2_transform = world
                .entry(e2)
                .unwrap()
                .into_component::<Transform>()
                .unwrap();

            let a1 = *e2_transform.global_matrix();
            let a2 = together(global_matrix1, local2.matrix());
            assert_eq!(a1, a2);
            a1
        };

        {
            let e3_transform = world
                .entry(e3)
                .unwrap()
                .into_component::<Transform>()
                .unwrap();

            let a1 = e3_transform.global_matrix();
            let a2 = together(global_matrix2, local3.matrix());
            assert_eq!(*a1, a2);
        };
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    #[allow(clippy::eq_op, clippy::zero_divided_by_zero)]
    fn nan_transform() {
        let (mut res, mut world, mut dispatcher) = transform_world();

        let mut local = Transform::default();
        // Release the indeterminate forms!
        local.set_translation_xyz(0.0 / 0.0, 0.0 / 0.0, 0.0 / 0.0);

        world.push((local,));

        dispatcher.execute(&mut world, &mut res);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn is_finite_transform() {
        let (mut res, mut world, mut dispatcher) = transform_world();

        let mut local = Transform::default();
        // Release the indeterminate forms!
        local.set_translation_xyz(1.0 / 0.0, 1.0 / 0.0, 1.0 / 0.0);
        world.push((local,));

        dispatcher.execute(&mut world, &mut res);
    }
}
