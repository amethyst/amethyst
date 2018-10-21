//! Provides a small simple tag component for identifying entities.

use amethyst_assets::{PrefabData, PrefabError};
use amethyst_core::specs::prelude::{
    Component, Entities, Entity, Join, NullStorage, ReadStorage, WriteStorage,
};
use std::marker::PhantomData;

/// Tag component that can be used with a custom type to tag entities for processing
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Tag<T> {
    _m: PhantomData<T>,
}

impl<T> Default for Tag<T> {
    fn default() -> Self {
        Tag { _m: PhantomData }
    }
}

impl<T> Component for Tag<T>
where
    T: Send + Sync + 'static,
{
    type Storage = NullStorage<Self>;
}

impl<'a, T> PrefabData<'a> for Tag<T>
where
    T: Clone + Send + Sync + 'static,
{
    type SystemData = WriteStorage<'a, Tag<T>>;
    type Result = ();

    fn add_to_entity(
        &self,
        entity: Entity,
        storage: &mut Self::SystemData,
        _: &[Entity],
    ) -> Result<(), PrefabError> {
        storage.insert(entity, self.clone()).map(|_| ())
    }
}

/// Utility lookup for tag components
#[derive(SystemData)]
pub struct TagFinder<'a, T>
where
    T: Send + Sync + 'static,
{
    /// The `EntitiesRes` from the ECS used to lookup tags.
    pub entities: Entities<'a>,
    /// The component storage for the tags being searched.
    pub tags: ReadStorage<'a, Tag<T>>,
}

impl<'a, T> TagFinder<'a, T>
where
    T: Send + Sync + 'static,
{
    /// Returns the first entity found with the tag in question.
    pub fn find(&self) -> Option<Entity> {
        (&*self.entities, &self.tags)
            .join()
            .map(|(entity, _)| entity)
            .next()
    }
}
