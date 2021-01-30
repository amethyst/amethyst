//! System that inserts [PreviousParent] components for entities that have [Transform] and [Parent]

use super::components::*;
use crate::ecs::*;

/// System that inserts [PreviousParent] components for entities that have [Transform] and [Parent]
#[derive(Debug)]
pub struct MissingPreviousParentSystem;

impl System for MissingPreviousParentSystem {
    fn build(self) -> Box<dyn ParallelRunnable> {
        Box::new(
            SystemBuilder::new("MissingPreviousParentSystem")
                // Entities with missing `PreviousParent`
                .with_query(
                    <(Entity, &Parent)>::query()
                        .filter(component::<Transform>() & !component::<PreviousParent>()),
                )
                .build(move |commands, world, _resource, query| {
                    // Add missing `PreviousParent` components
                    for (entity, _parent) in query.iter(world) {
                        log::trace!("Adding missing PreviousParent to {:?}", entity);
                        commands.add_component(*entity, PreviousParent(None));
                    }
                }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn previous_parent_added() {
        let mut resources = Resources::default();
        let mut world = World::default();

        let mut schedule = Schedule::from(vec![
            systems::Step::Systems(systems::Executor::new(vec![
                MissingPreviousParentSystem.build()
            ])),
            systems::Step::FlushCmdBuffers,
        ]);

        let e1 = world.push((Transform::default(),));

        let e2 = world.push((Transform::default(), Parent(e1)));

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(e1)
                .unwrap()
                .get_component::<PreviousParent>()
                .is_ok(),
            false
        );

        assert_eq!(
            world
                .entry(e2)
                .unwrap()
                .get_component::<PreviousParent>()
                .is_ok(),
            true
        );
    }
}
