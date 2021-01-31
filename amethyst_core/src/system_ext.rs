//! Extension system utilities
//!
//! This module contains useful functions to extend and transform existing systems.

use legion::{
    storage::ComponentTypeId,
    systems::{
        CommandBuffer, ParallelRunnable, Resource, ResourceSet, ResourceTypeId, Runnable, SystemId,
        UnsafeResources,
    },
    world::{ArchetypeAccess, WorldId},
    Read, World,
};

/// Make a system pausable by tying it to a specific value of a resource.
///
/// When the value of the resource differs from `value` the system is considered "paused",
/// and the `run` method of the pausable system will not be called.
///
/// # Notes
///
/// Special care must be taken not to read from an `EventChannel` with pausable systems.
/// Since `run` is never called, there is no way to consume the reader side of a channel, and
/// it may grow indefinitely leaking memory while the system is paused.
///
/// # Examples
/// ```
/// use std::ops::Deref;
///
/// use amethyst::core::{
///     dispatcher::DispatcherBuilder,
///     ecs::{system, ParallelRunnable, Resources, Schedule, System, SystemBuilder, World},
///     system_ext::pausable,
/// };
///
/// #[derive(PartialEq)]
/// enum CurrentState {
///     Disabled,
///     Enabled,
/// }
///
/// struct TestSystem;
///
/// impl System for TestSystem {
///     fn build(self) -> Box<dyn ParallelRunnable> {
///         Box::new(pausable(
///             SystemBuilder::new("TestSystem")
///                 .write_resource::<u32>()
///                 .build(move |_commands, _world, resources, _| {
///                     **resources += 1;
///                 }),
///             CurrentState::Enabled,
///         ))
///     }
/// }
///
/// let mut world = World::default();
/// let mut resources = Resources::default();
///
/// let mut dispatcher = DispatcherBuilder::default()
///     .add_system(TestSystem)
///     .build(&mut world, &mut resources)
///     .unwrap();
///
/// // we only expect the u32 resource to be modified _if_ the system is enabled,
/// // the system should only be enabled on CurrentState::Enabled.
/// resources.insert(1u32);
/// resources.insert(CurrentState::Disabled);
/// dispatcher.execute(&mut world, &mut resources);
/// assert_eq!(1, *resources.get::<u32>().unwrap().deref());
///
/// resources.insert(CurrentState::Enabled);
/// dispatcher.execute(&mut world, &mut resources);
/// assert_eq!(2, *resources.get::<u32>().unwrap().deref());
/// ```
pub fn pausable<V>(runnable: impl ParallelRunnable, value: V) -> Pausable<impl ParallelRunnable, V>
where
    V: Resource + PartialEq,
{
    let (resource_reads, _) = runnable.reads();
    let resource_reads = resource_reads
        .iter()
        .copied()
        .chain(std::iter::once(ResourceTypeId::of::<V>()))
        .collect::<Vec<_>>();
    Pausable {
        system: runnable,
        value,
        resource_reads,
    }
}

/// A system that is enabled when `V` has a specific value.
///
/// This is created using the [`pausable`] method.
///
/// [`pausable`]: fn.pausable.html
#[derive(Debug)]
pub struct Pausable<S, V> {
    system: S,
    value: V,
    resource_reads: Vec<ResourceTypeId>,
}

impl<S, V> Runnable for Pausable<S, V>
where
    S: Runnable,
    V: Resource + PartialEq,
{
    // Default passthrough impls
    fn name(&self) -> Option<&SystemId> {
        self.system.name()
    }

    fn reads(&self) -> (&[ResourceTypeId], &[ComponentTypeId]) {
        let (_, components) = self.system.reads();
        // Return our local copy of systems resources that's been appended with permission for Read<V>
        (&self.resource_reads[..], components)
    }

    fn writes(&self) -> (&[ResourceTypeId], &[ComponentTypeId]) {
        self.system.writes()
    }

    fn prepare(&mut self, world: &World) {
        self.system.prepare(world)
    }

    fn accesses_archetypes(&self) -> &ArchetypeAccess {
        self.system.accesses_archetypes()
    }

    unsafe fn run_unsafe(&mut self, world: &World, resources: &UnsafeResources) {
        let resources_static = &*(resources as *const UnsafeResources);
        let resource_to_check = Read::<V>::fetch_unchecked(resources_static);

        if self.value != *resource_to_check {
            return;
        }

        self.system.run_unsafe(world, resources);
    }

    fn command_buffer_mut(&mut self, world: WorldId) -> Option<&mut CommandBuffer> {
        self.system.command_buffer_mut(world)
    }
}

#[cfg(test)]
mod test {
    use legion::{Resources, SystemBuilder};

    use super::*;
    use crate::{
        dispatcher::{DispatcherBuilder, System},
        ecs::ParallelRunnable,
    };

    #[derive(PartialEq)]
    enum CurrentState {
        Disabled,
        Enabled,
    }

    struct TestSystem;

    impl System for TestSystem {
        fn build(self) -> Box<dyn ParallelRunnable> {
            Box::new(pausable(
                SystemBuilder::new("TestSystem")
                    .write_resource::<u32>()
                    .build(move |_commands, _world, resources, _| {
                        **resources += 1;
                    }),
                CurrentState::Enabled,
            ))
        }
    }

    #[test]
    fn should_not_pause_if_resource_match_value() {
        let mut resources = Resources::default();
        let mut world = World::default();
        resources.insert(0u32);
        resources.insert(CurrentState::Enabled);

        let mut dispatcher = DispatcherBuilder::default()
            .add_system(TestSystem)
            .build(&mut world, &mut resources)
            .unwrap();

        assert_eq!(0, *resources.get::<u32>().unwrap());
        dispatcher.execute(&mut world, &mut resources);
        assert_eq!(1, *resources.get::<u32>().unwrap());
        dispatcher.execute(&mut world, &mut resources);
        assert_eq!(2, *resources.get::<u32>().unwrap());
    }

    #[test]
    fn should_pause_if_resource_does_not_match_value() {
        let mut resources = Resources::default();
        let mut world = World::default();
        resources.insert(0u32);
        resources.insert(CurrentState::Enabled);

        let mut dispatcher = DispatcherBuilder::default()
            .add_system(TestSystem)
            .build(&mut world, &mut resources)
            .unwrap();

        assert_eq!(0, *resources.get::<u32>().unwrap());
        dispatcher.execute(&mut world, &mut resources);
        assert_eq!(1, *resources.get::<u32>().unwrap());

        resources.insert(CurrentState::Disabled);

        dispatcher.execute(&mut world, &mut resources);
        assert_eq!(1, *resources.get::<u32>().unwrap());
    }
}
