//! Provides utilities to remove large amounts of entities with a single command.

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::world::EntitiesRes;
use amethyst_core::specs::{Component, DenseVecStorage, Entity, Join, ReadStorage, WriteStorage};
use std::fmt::Debug;
use std::result::Result;

/// A marker `Component` used to remove entities and clean up your scene.
/// The generic parameter `I` is the type of id you want to use.
/// Generally an int or an enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Removal<I>
where
    I: Debug + Clone,
{
    id: I,
}

impl<I> Removal<I>
where
    I: Debug + Clone,
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

/// The prefab allowing to easily add a `Removal` `Component` to an entity.
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct RemovalPrefab<I: Debug> {
    id: I,
}

impl<'a, I> PrefabData<'a> for RemovalPrefab<I>
where
    I: PartialEq + Debug + Clone + Send + Sync + 'static,
{
    type SystemData = (WriteStorage<'a, Removal<I>>,);
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        system_data: &mut Self::SystemData,
        _entities: &[Entity],
    ) -> Result<(), PrefabError> {
        system_data
            .0
            .insert(entity, Removal::new(self.id.clone()))?;
        Ok(())
    }
}

/// Removes all entities that have the `Removal<I>` component with the specified removal_id.
pub fn exec_removal<I>(
    entities: &EntitiesRes,
    removal_storage: &ReadStorage<Removal<I>>,
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
