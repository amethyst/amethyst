//! Provides utilities to remove large amounts of entities with a single command.

use std::fmt::Debug;

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::{
    world::EntitiesRes, Component, DenseVecStorage, Entity, Join, ReadStorage, WriteStorage,
};
use amethyst_derive::PrefabData;

use log::error;
use serde::{Deserialize, Serialize};

/// A marker `Component` used to remove entities and clean up your scene.
/// The generic parameter `I` is the type of id you want to use.
/// Generally an int or an enum.
#[derive(Debug, Clone, Serialize, Deserialize, PrefabData)]
#[prefab(Component)]
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

impl<I> Component for Removal<I>
where
    I: Debug + Clone + Send + Sync + 'static,
{
    type Storage = DenseVecStorage<Self>;
}

/// Removes all entities that have the `Removal<I>` component with the specified removal_id.
pub fn exec_removal<I>(
    entities: &EntitiesRes,
    removal_storage: &ReadStorage<'_, Removal<I>>,
    removal_id: I,
) where
    I: Debug + Clone + PartialEq + Send + Sync + 'static,
{
    for (e, _) in (&*entities, removal_storage)
        .join()
        .filter(|(_, r)| r.id == removal_id)
    {
        if let Err(err) = entities.delete(e) {
            error!("Failed to delete entity during exec_removal: {:?}", err);
        }
    }
}
