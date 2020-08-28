//! Extension system utilities
//!
//! This module contains useful functions to extend and transform existing systems.

use crate::legion::{
    storage::ComponentTypeId,
    systems::{
        CommandBuffer, Resource, ResourceSet, ResourceTypeId, Runnable, SystemId, UnsafeResources,
    },
    world::{ArchetypeAccess, World, WorldId},
    Read,
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
/// ```rust
/// use legion::{system, Schedule, World, Resources};
///
/// #[derive(PartialEq)]
/// enum CurrentState {
///     Disabled,
///     Enabled,
/// }
///
/// #[system]
/// fn add_number(#[state] n: &u32, #[resource] sum: &mut u32) {
///     *sum += n;
/// }
///
/// let mut schedule = Schedule::builder()
///     .add_system(add_number_system(1))
///     .add_system(pauseable(add_number_system(2), CurrentState::Enabled))
///     .build();
///
/// let mut world = World::default();
/// let mut resources = Resources::default();
///
/// // we only expect the u32 resource to be modified _if_ the system is enabled,
/// // the system should only be enabled on CurrentState::Enabled.
/// resources.insert(0u32);
/// resources.insert(CurrentState::Disabled);
/// schedule.execute(&mut world, &mut resources);
/// assert_eq!(1, resources.get::<u32>().unwrap());
///
/// resources.insert(0u32);
/// resources.insert(CurrentState::Enabled);
/// schedule.execute(&mut world, &mut resources);
/// assert_eq!(1 + 2, resources.get::<u32>().unwrap());
/// ```
pub fn pauseable<V>(runnable: impl Runnable, value: V) -> Pauseable<impl Runnable, V>
where
    V: Resource + PartialEq,
{
    let (resource_reads, _) = runnable.reads();
    let resource_reads = resource_reads
        .iter()
        .copied()
        .chain(std::iter::once(ResourceTypeId::of::<V>()))
        .collect::<Vec<_>>();
    Pauseable {
        system: runnable,
        value,
        resource_reads,
    }
}

/// A system that is enabled when `V` has a specific value.
///
/// This is created using the [`SystemExt::pausable`] method.
///
/// [`SystemExt::pausable`]: trait.SystemExt.html#tymethod.pausable
#[derive(Debug)]
pub struct Pauseable<S, V> {
    system: S,
    value: V,
    resource_reads: Vec<ResourceTypeId>,
}

impl<S, V> Runnable for Pauseable<S, V>
where
    S: Runnable,
    V: Resource + PartialEq,
{
    fn reads(&self) -> (&[ResourceTypeId], &[ComponentTypeId]) {
        let (_, components) = self.system.reads();
        // Return our local copy of systems resources that's been appended with permission for Read<V>
        (&self.resource_reads[..], components)
    }

    unsafe fn run_unsafe(&mut self, world: &World, resources: &UnsafeResources) {
        let resources_static = &*(resources as *const UnsafeResources);
        let resource_to_check = Read::<V>::fetch_unchecked(resources_static);

        if self.value != *resource_to_check {
            return;
        }

        self.system.run_unsafe(world, resources);
    }

    // Default passthrough impls
    fn name(&self) -> Option<&SystemId> {
        self.system.name()
    }

    fn prepare(&mut self, world: &World) {
        self.system.prepare(world)
    }

    fn accesses_archetypes(&self) -> &ArchetypeAccess {
        self.system.accesses_archetypes()
    }

    fn writes(&self) -> (&[ResourceTypeId], &[ComponentTypeId]) {
        self.system.writes()
    }

    fn command_buffer_mut(&mut self, world: WorldId) -> Option<&mut CommandBuffer> {
        self.system.command_buffer_mut(world)
    }
}
