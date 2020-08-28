#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

use crate::legion::{
    storage::ComponentTypeId,
    systems::{
        CommandBuffer, QuerySet, Resource, ResourceSet, ResourceTypeId, Runnable, System, SystemFn,
        SystemId, UnsafeResources,
    },
    world::*,
    Read,
};

/// Extension functionality associated systems.
pub trait SystemExt {
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
    ///
    /// ```rust
    /// use amethyst::{
    ///     ecs::{System, Write, system},
    ///     shred::DispatcherBuilder,
    ///     prelude::*,
    /// };
    ///
    /// #[derive(PartialEq)]
    /// enum CurrentState {
    ///     Disabled,
    ///     Enabled,
    /// }
    ///
    /// impl Default for CurrentState {
    ///     fn default() -> Self {
    ///         CurrentState::Disabled
    ///     }
    /// }
    ///
    /// struct AddNumber(u32);
    ///
    /// impl<'s> System<'s> for AddNumber {
    ///     type SystemData = Write<'s, u32>;
    ///
    ///     fn run(&mut self, mut number: Self::SystemData) {
    ///         *number += self.0;
    ///     }
    /// }
    ///
    /// let mut world = World::new();
    ///
    /// let mut dispatcher = DispatcherBuilder::default()
    ///     .with(AddNumber(1), "set_number", &[])
    ///     .with(AddNumber(2).pausable(CurrentState::Enabled), "set_number_2", &[])
    ///     .build();
    ///
    /// dispatcher.setup(&mut world);
    /// 
    ///
    /// // we only expect the u32 resource to be modified _if_ the system is enabled,
    /// // the system should only be enabled on CurrentState::Enabled.
    ///
    /// *world.write_resource() = 0u32;
    /// dispatcher.dispatch(&mut world);
    /// assert_eq!(1, *world.read_resource::<u32>());
    ///
    /// *world.write_resource() = 0u32;
    /// *world.write_resource() = CurrentState::Enabled;
    /// dispatcher.dispatch(&mut world);
    /// assert_eq!(1 + 2, *world.read_resource::<u32>());
    /// ```
    fn pausable<V>(self, value: V) -> Pausable<Self, V>
    where
        Self: Sized,
        V: Resource + Default + PartialEq;
}

impl<S> SystemExt for S
where
    S: Runnable,
{
    fn pausable<V>(self, value: V) -> Pausable<Self, V>
    where
        Self: Sized,
        V: Resource + Default + PartialEq,
    {
        /* 
         * We're required to provide a &[ResourceTypeId] which means we need to append our V to the current 
         * ResourceSet while also allocating a continuous slice of memory that can be returned by reference.
         * This precludes doing something like constructing the vec in Runnable.reads() below as we'd have to 
         * return a reference to the local Vec that's being deallocated.
         */
        let (resource_reads, _) = self.reads();
        let resource_reads = resource_reads
            .into_iter()
            .map(|id| *id)
            .chain(std::iter::once(ResourceTypeId::of::<V>()))
            .collect::<Vec<_>>();
        Pausable {
            system: self,
            value,
            resource_reads,
        }
    }
}

/// A system that is enabled when `V` has a specific value.
///
/// This is created using the [`SystemExt::pausable`] method.
///
/// [`SystemExt::pausable`]: trait.SystemExt.html#tymethod.pausable
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
/* WIP Example, not currently compiling */
use legion::{system, Schedule};

#[derive(PartialEq)]
enum CurrentState {
    Disabled,
    Enabled,
}

#[system]
fn add_number(#[state] n: &u32, #[resource] sum: &mut u32) {
    *sum += n;
}

fn emample() {
    let add_2 = add_number_system(2);

    Schedule::builder()
        .add_system(add_number_system(1))
        .add_system(add_2.pauseable(CurrentState::Enabled));
}