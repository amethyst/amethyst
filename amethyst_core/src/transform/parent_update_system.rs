//! System that generates [Children] components for entities that are targeted by [Parent] component.

use std::collections::HashMap;

use smallvec::SmallVec;

use super::components::*;
use crate::ecs::*;

/// System that generates [Children] components for entities that are targeted by [Parent] component.
#[derive(Debug)]
pub struct ParentUpdateSystem;

impl System for ParentUpdateSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("ParentUpdateSystem")
                // Entities with a removed `Parent`
                .with_query(<(Entity, &PreviousParent)>::query().filter(!component::<Parent>()))
                // Entities with a changed `Parent`
                .with_query(
                    <(Entity, &Parent, &mut PreviousParent)>::query()
                        .filter(component::<Transform>() & maybe_changed::<Parent>()),
                )
                // Deleted Parents (ie Entities with `Children` and without a `Transform`).
                .with_query(<(Entity, &Children)>::query().filter(!component::<Transform>()))
                .write_component::<Children>()
                .build(move |commands, world, _resource, queries| {
                    // Entities with a missing `Parent` (ie. ones that have a `PreviousParent`), remove
                    // them from the `Children` of the `PreviousParent`.
                    let (ref mut left, ref mut right) = world.split::<Write<Children>>();
                    for (entity, previous_parent) in queries.0.iter(right) {
                        log::trace!("Parent was removed from {:?}", entity);
                        if let Some(previous_parent_entity) = previous_parent.0 {
                            if let Some(previous_parent_children) = left
                                .entry_mut(previous_parent_entity)
                                .ok()
                                .and_then(|entry| entry.into_component_mut::<Children>().ok())
                            {
                                log::trace!(
                                    " > Removing {:?} from it's prev parent's children",
                                    entity
                                );
                                previous_parent_children.0.retain(|e| e != entity);
                            }
                        }
                    }

                    // Tracks all newly created `Children` Components this frame.
                    let mut children_additions =
                        HashMap::<Entity, SmallVec<[Entity; 8]>>::with_capacity(16);

                    // Entities with a changed Parent (that also have a PreviousParent, even if None)
                    for (entity, parent, previous_parent) in queries.1.iter_mut(right) {
                        log::trace!("Parent changed for {:?}", entity);

                        // If the `PreviousParent` is not None.
                        if let Some(previous_parent_entity) = previous_parent.0 {
                            // New and previous point to the same Entity, carry on, nothing to see here.
                            if previous_parent_entity == parent.0 {
                                log::trace!(" > But the previous parent is the same, ignoring...");
                                continue;
                            }

                            // Remove from `PreviousParent.Children`.
                            if let Some(previous_parent_children) = left
                                .entry_mut(previous_parent_entity)
                                .ok()
                                .and_then(|entry| entry.into_component_mut::<Children>().ok())
                            {
                                log::trace!(" > Removing {:?} from prev parent's children", entity);
                                previous_parent_children.0.retain(|e| e != entity);
                            }
                        }

                        // Set `PreviousParent = Parent`.
                        *previous_parent = PreviousParent(Some(parent.0));

                        // Add to the parent's `Children` (either the real component, or
                        // `children_additions`).
                        log::trace!("Adding {:?} to it's new parent {:?}", entity, parent.0);
                        if let Some(new_parent_children) = left
                            .entry_mut(parent.0)
                            .ok()
                            .and_then(|entry| entry.into_component_mut::<Children>().ok())
                        {
                            // This is the parent
                            log::trace!(
                                " > The new parent {:?} already has a `Children`, adding to it.",
                                parent.0
                            );
                            new_parent_children.0.push(*entity);
                        } else {
                            // The parent doesn't have a children entity, lets add it
                            log::trace!(
                                "The new parent {:?} doesn't yet have `Children` component.",
                                parent.0
                            );
                            children_additions
                                .entry(parent.0)
                                .or_insert_with(Default::default)
                                .push(*entity);
                        }
                    }

                    // Deleted `Parents` (ie. Entities with a `Children` but no `Transform`).
                    for (entity, children) in queries.2.iter(world) {
                        log::trace!("The entity {:?} doesn't have a Transform", entity);
                        if children_additions.remove(&entity).is_none() {
                            log::trace!(" > It needs to be remove from the ECS.");
                            for child_entity in children.0.iter() {
                                commands.remove_component::<Parent>(*child_entity);
                                commands.remove_component::<PreviousParent>(*child_entity);
                            }
                            commands.remove_component::<Children>(*entity);
                        } else {
                            log::trace!(" > It was a new addition, removing it from additions map");
                        }
                    }

                    // Flush the `children_additions` to the command buffer. It is stored separate to
                    // collect multiple new children that point to the same parent into the same
                    // SmallVec, and to prevent redundant add+remove operations.
                    children_additions.iter().for_each(|(k, v)| {
                        log::trace!(
                            "Flushing: Entity {:?} adding `Children` component {:?}",
                            k,
                            v
                        );
                        commands.add_component(*k, Children::with(v));
                    });
                }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transform::MissingPreviousParentSystem;

    #[test]
    fn correct_children() {
        let mut resources = Resources::default();
        let mut world = World::default();

        let mut schedule = Schedule::from(vec![
            systems::Step::Systems(systems::Executor::new(vec![
                MissingPreviousParentSystem.build()
            ])),
            systems::Step::FlushCmdBuffers,
            systems::Step::Systems(systems::Executor::new(vec![ParentUpdateSystem.build()])),
            systems::Step::FlushCmdBuffers,
        ]);

        // Add parent entities
        let parent = world.push((Transform::default(),));
        let children = world.extend(vec![(Transform::default(),), (Transform::default(),)]);
        let (e1, e2) = (children[0], children[1]);

        // Parent `e1` and `e2` to `parent`.
        world.entry(e1).unwrap().add_component(Parent(parent));
        world.entry(e2).unwrap().add_component(Parent(parent));

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(parent)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e1, e2]
        );

        // Parent `e1` to `e2`.
        world
            .entry_mut(e1)
            .unwrap()
            .get_component_mut::<Parent>()
            .unwrap()
            .0 = e2;

        // Run the systems
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(parent)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e2]
        );

        assert_eq!(
            world
                .entry(e2)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e1]
        );

        world.remove(e1);

        // Run the systems
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(parent)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e2]
        );
    }
}
