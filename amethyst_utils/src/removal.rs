//! Provides utilities to remove large amounts of entities with a single command.

use std::fmt::Debug;

use amethyst_core::ecs::*;
use serde::{Deserialize, Serialize};

/// A marker `Component` used to remove entities and clean up your scene.
/// The generic parameter `I` is the type of id you want to use.
/// Generally an int or an enum.
///
/// # Example
///
/// ```
/// # use amethyst::core::ecs::*;
/// # use amethyst::utils::removal::*;
/// let mut world = World::default();
/// let mut buffer = CommandBuffer::new(&mut world);
///
/// #[derive(Clone, Debug, PartialEq)]
/// enum RemovalId {
///     Something,
///     Something2,
/// }
///
/// world.push((Removal::new(RemovalId::Something),));
/// world.push((Removal::new(RemovalId::Something2),));
///
/// // Remove all entities with the RemovalId value of Something.
/// amethyst::utils::removal::exec_removal(
///     &mut buffer,
///     &mut (&mut world).into(),
///     RemovalId::Something,
/// );
///
/// // Force the world to be up to date. This is normally called automatically at the end of the
/// // frame by amethyst.
/// buffer.flush(&mut world);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Removal<I>
where
    I: Debug + Clone + Send + Sync + 'static,
{
    id: I,
}

impl<I> Removal<I>
where
    I: Debug + Clone + Send + Sync + 'static,
{
    /// Creates a new `Removal` component with the specified id.
    pub fn new(id: I) -> Self {
        Removal { id }
    }
}

/// Removes all entities that have the `Removal<I>` component with the specified removal_id.
pub fn exec_removal<I>(commands: &mut CommandBuffer, subworld: &mut SubWorld<'_>, removal_id: I)
where
    I: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    let mut removal_query = <(Entity, &Removal<I>)>::query();
    for (ent, _) in removal_query
        .iter_mut(subworld)
        .filter(|(_, rm)| rm.id == removal_id)
    {
        commands.remove(*ent);
    }
}

/// Adds a `Removal` component with the specified id to the specified entity.
pub fn add_removal_to_entity<T: PartialEq + Clone + Debug + Send + Sync + 'static>(
    world: &mut World,
    entity: Entity,
    id: T,
) {
    world
        .entry(entity)
        .map(|mut entry| entry.add_component(Removal::new(id)))
        .unwrap_or_else(|| {
            panic!(
                "Failed to insert `Removal` component id to entity: {:?}.",
                entity,
            )
        });
}
